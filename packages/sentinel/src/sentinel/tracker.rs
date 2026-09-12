// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Subspace tracker — the core online SVD engine.
//!
//! Maintains a low-rank model of "normal" via streaming thin SVD
//! with exponential forgetting.  Scores each new batch along four
//! axes (novelty, displacement, surprise, coherence), then evolves
//! the model to incorporate the new data.
//!
//! See `docs/algorithm.md` §4.2 for the five-phase core loop and
//! §5 for the four scoring axes.
//!
//! This is the only module that depends on `faer`.

use faer::Mat;
use torrust_mudlark::Accumulator;

use crate::config::SentinelConfig;
use crate::ewma::EwmaStats;
use crate::report::{AnomalyScores, SampleScore, ScoreDistribution, ScoringGeometry, TrackerMaturity, TrackerReport};
use crate::sentinel::cusum::CusumAccumulator;

// ─── Per-axis baseline ──────────────────────────────────────

/// Fast EWMA (z-scores) + CUSUM (drift detection) for one scoring axis.
#[derive(Debug, Clone)]
struct AxisBaseline {
    fast: EwmaStats,
    cusum: CusumAccumulator,
    /// Clip-pressure EWMA: ρ̄ ∈ [0, 1]  (§ALGO S-6.4).
    clip_pressure: f64,
}

impl AxisBaseline {
    const fn new(fast_decay: f64, slow_decay: f64) -> Self {
        Self {
            fast: EwmaStats::new(fast_decay),
            cusum: CusumAccumulator::new(slow_decay),
            clip_pressure: 0.0,
        }
    }

    /// Destroy all learned state — return to the freshly-constructed
    /// state with cold EWMA and zeroed CUSUM.
    const fn reset_cold(&mut self) {
        self.fast.reset_cold();
        self.cusum.reset_cold();
        self.clip_pressure = 0.0;
    }

    /// Seed the CUSUM's slow EWMA from this axis's fast EWMA.
    ///
    /// Closes the fast-slow gap after noise injection (ADR-S-013
    /// §6b, Option C).
    const fn seed_cusum_slow_from_fast(&mut self) {
        self.cusum.seed_slow_from(&self.fast);
    }
}

// ─── SubspaceTracker ────────────────────────────────────────

/// Low-rank subspace model with online learning and four-axis scoring.
///
/// Each analysis cell gets one of these.  The tracker
/// accepts centred suffix observation slices via [`observe`](Self::observe)
/// and returns a [`TrackerReport`].
///
/// The tracker has zero knowledge of cell identity, observation types,
/// or the host's domain.  It operates on `&[&[f64]]` — a batch of
/// d-dimensional centred bit slices.
#[derive(Debug, Clone)]
pub struct SubspaceTracker {
    /// Suffix width: dimensionality of the working space (128 − depth).
    dim: usize,

    /// Hard ceiling on rank: `min(dim, max_rank)`.
    cap: usize,

    /// Current active rank (number of basis vectors in use).
    rank: usize,

    /// Observation step counter (for rank adaptation timing).
    step: u64,

    // ── Subspace state ──────────────────────────────────
    /// Orthonormal basis, shape `(dim, cap)`.  Only columns `[:rank]` are active.
    basis: Mat<f64>,

    /// Singular values, length `cap`.  Only `[:rank]` are meaningful.
    sigmas: Vec<f64>,

    // ── Latent distribution ─────────────────────────────
    /// EWMA mean of latent coordinates, length `cap`.
    lat_mean: Vec<f64>,

    /// EWMA variance of latent coordinates, length `cap`.
    lat_var: Vec<f64>,

    /// Upper-triangle cross-correlation, flat-packed.
    /// `cross_corr[tri_idx(j, l, cap)]` for `j < l` tracks EWMA of `zⱼ · zₗ`.
    ///
    /// Length: `cap * (cap - 1) / 2`.  When rank increases, new entries
    /// are already zero (the full triangle is pre-allocated at construction).
    /// When rank decreases, outer entries are ignored but preserved (§4.2 Phase 3).
    cross_corr: Vec<f64>,

    // ── Score baselines (one per axis) ──────────────────
    novelty_bl: AxisBaseline,
    displacement_bl: AxisBaseline,
    surprise_bl: AxisBaseline,
    coherence_bl: AxisBaseline,

    // ── Maturity tracking ───────────────────────────────
    real_observations: u64,
    noise_observations: u64,
    noise_influence: f64,

    // ── Config snapshots ────────────────────────────────
    forgetting_factor: f64,
    energy_threshold: f64,
    rank_update_interval: u64,
    eps: f64,
    per_sample_scores: bool,
    cusum_allowance_sigmas: f64,
    clip_sigmas: f64,
    /// Clip-pressure EWMA decay factor (`λ_ρ`, §ALGO S-6.4).
    clip_pressure_decay: f64,
    svd_strategy: crate::maths::SvdStrategy,
}

impl SubspaceTracker {
    /// Create a new tracker for the given suffix width.
    ///
    /// The initial basis is identity-like columns (not random).
    /// Noise injection will diversify it before real traffic arrives.
    pub fn new<V: Accumulator>(dim: usize, cfg: &SentinelConfig<V>, slow_decay: f64) -> Self {
        debug_assert!(
            dim >= crate::MIN_TRACKER_DIM,
            "SubspaceTracker::new() called with dim={dim}, \
             expected >= {} (caller should have filtered)",
            crate::MIN_TRACKER_DIM,
        );

        let cap = dim.min(cfg.max_rank);

        // Identity-like basis: column j has a 1.0 at row j.
        let mut basis = Mat::zeros(dim, cap);
        for j in 0..cap.min(dim) {
            basis[(j, j)] = 1.0;
        }

        let fast_decay = cfg.forgetting_factor;

        Self {
            dim,
            cap,
            rank: 1,
            step: 0,
            basis,
            sigmas: vec![0.01; cap],
            lat_mean: vec![0.0; cap],
            lat_var: vec![1.0; cap],
            cross_corr: vec![0.0; cap * (cap.saturating_sub(1)) / 2],
            novelty_bl: AxisBaseline::new(fast_decay, slow_decay),
            displacement_bl: AxisBaseline::new(fast_decay, slow_decay),
            surprise_bl: AxisBaseline::new(fast_decay, slow_decay),
            coherence_bl: AxisBaseline::new(fast_decay, slow_decay),
            real_observations: 0,
            noise_observations: 0,
            noise_influence: 1.0,
            forgetting_factor: cfg.forgetting_factor,
            energy_threshold: cfg.energy_threshold,
            rank_update_interval: cfg.rank_update_interval,
            eps: cfg.eps,
            per_sample_scores: cfg.per_sample_scores,
            cusum_allowance_sigmas: cfg.cusum_allowance_sigmas,
            clip_sigmas: cfg.clip_sigmas,
            clip_pressure_decay: cfg.clip_pressure_decay,
            svd_strategy: cfg.svd_strategy,
        }
    }

    /// Process a batch of centred observation slices and return a report.
    ///
    /// Each inner slice in `rows` has length `self.dim`.
    /// Scoring happens against the *prior* model, then the model evolves.
    ///
    /// `is_noise` controls maturity bookkeeping — noise observations
    /// don't count as real.
    #[allow(clippy::many_single_char_names)] // mathematical notation matching the spec
    pub fn observe(&mut self, rows: &[&[f64]], depth: u8, is_noise: bool) -> TrackerReport {
        let _observe_span = tracing::debug_span!("observe", depth, is_noise, b = rows.len(), k = self.rank).entered();

        let b = rows.len();
        let d = self.dim;
        let k = self.rank;
        let eps = self.eps;

        // Build X matrix (b × d).
        let x = Self::build_matrix(rows, b, d);

        // ── Phase 1: Score against the prior model ──────
        let p1_guard = tracing::debug_span!("phase1_score").entered();
        // Capture Z from this phase for Phase 3 (latent evolution uses
        // the prior-basis projection, not the post-evolution basis).
        let u_k = self.basis.subcols(0, k);
        let z = &x * u_k; // (b × k)
        let x_hat = &z * u_k.transpose(); // (b × d)
        let residual = &x - &x_hat; // (b × d)

        let (nov_scores, disp_scores, surp_scores, coh_scores) = self.compute_scores(&z, &residual, b, d, k, eps);

        // Build per-sample structs only when enabled (avoids 4 z-score
        // calls per sample in the common disabled case).
        let per_sample = if self.per_sample_scores {
            Some(self.build_per_sample(&nov_scores, &disp_scores, &surp_scores, &coh_scores, eps))
        } else {
            None
        };
        drop(p1_guard);

        // ── Phase 2: Evolve subspace (streaming SVD) ─────
        // Uses the maths module dispatch: the selected SvdStrategy
        // runs, and in debug builds the other strategy also runs
        // with results compared via debug_assert (ADR-S-016).
        self.evolve_subspace(&z, &residual, k);

        // ── Phase 3: Evolve latent distribution ─────────
        let p3_guard = tracing::debug_span!("phase3_latent").entered();
        // Uses Z from Phase 1 (prior basis), not the updated basis.
        self.evolve_latent(&z, k);
        drop(p3_guard);

        // ── Phase 4: Update score baselines and CUSUM ───
        //
        // Unified clip + clip-pressure EWMA (§ALGO S-6.1.1, §ALGO S-6.4).
        //
        // Each axis computes its own effective clip width from:
        //   p = max(η, ρ̄)           — noise influence OR clip pressure
        //   n_σ_eff = n_σ · (1 + p / (1 − p + ε))
        //
        // A single shared clip filter (from the fast EWMA ceiling) is
        // applied, and both the fast EWMA and CUSUM slow EWMA receive
        // the same retained set.  The per-axis clip-pressure ρ̄ is
        // updated from the fraction of samples clipped.
        //
        // At ρ̄ = 0 this reduces to the old η-only formula:
        //   n_σ_eff = n_σ + n_σ · η / (1 − η + ε)
        let allowance = self.cusum_allowance_sigmas;
        let eta = self.noise_influence;
        let clip_sigmas = self.clip_sigmas;
        let cp_decay = self.clip_pressure_decay;

        let novelty_dist = Self::update_axis(
            &mut self.novelty_bl,
            &nov_scores,
            eps,
            allowance,
            clip_sigmas,
            eta,
            cp_decay,
            true,
        );
        let displacement_dist = Self::update_axis(
            &mut self.displacement_bl,
            &disp_scores,
            eps,
            allowance,
            clip_sigmas,
            eta,
            cp_decay,
            true,
        );
        let surprise_dist = Self::update_axis(
            &mut self.surprise_bl,
            &surp_scores,
            eps,
            allowance,
            clip_sigmas,
            eta,
            cp_decay,
            true,
        );
        // Coherence does not exist at k < 2 (no pairs).  Scores are
        // identically zero, so the baseline must not evolve — it stays
        // cold until k reaches 2, where the first real values enter
        // through the cold→warm path.  If rank later drops back below
        // 2, adapt_rank() destroys the coherence baseline entirely.
        let coherence_dist = Self::update_axis(
            &mut self.coherence_bl,
            &coh_scores,
            eps,
            allowance,
            clip_sigmas,
            eta,
            cp_decay,
            k >= 2,
        );

        // ── Phase 5: Adapt rank ─────────────────────────
        self.step += 1;
        if self.step.is_multiple_of(self.rank_update_interval) {
            self.adapt_rank();
        }

        // ── Maturity bookkeeping ────────────────────────
        self.update_maturity(b, is_noise);

        TrackerReport {
            depth,
            rank: self.rank,
            energy_ratio: self.energy_ratio(),
            top_singular_value: self.sigmas.first().copied().unwrap_or(0.0),
            scores: AnomalyScores {
                novelty: novelty_dist,
                displacement: displacement_dist,
                surprise: surprise_dist,
                coherence: coherence_dist,
            },
            maturity: self.maturity(),
            geometry: self.scoring_geometry(),
            per_sample,
        }
    }

    /// Reset all CUSUM accumulators to zero (post-noise-injection).
    pub const fn reset_cusum(&mut self) {
        self.novelty_bl.cusum.reset();
        self.displacement_bl.cusum.reset();
        self.surprise_bl.cusum.reset();
        self.coherence_bl.cusum.reset();
    }

    /// Zero clip-pressure EWMA on all four axes (§ALGO S-11.4).
    ///
    /// Prevents warm-up contamination from echoing into production
    /// scoring.  Called after noise injection completes, alongside
    /// [`reset_cusum`](Self::reset_cusum).
    pub const fn reset_clip_pressure(&mut self) {
        self.novelty_bl.clip_pressure = 0.0;
        self.displacement_bl.clip_pressure = 0.0;
        self.surprise_bl.clip_pressure = 0.0;
        self.coherence_bl.clip_pressure = 0.0;
    }

    /// Seed each axis's CUSUM slow EWMA from the corresponding fast EWMA.
    ///
    /// Closes the fast-slow gap after noise injection (ADR-S-013
    /// §6b, Option C).  Called **before** [`reset_cusum`](Self::reset_cusum)
    /// so the slow baselines start from the fast EWMA's converged
    /// values.  The CUSUM accumulators are then zeroed, beginning
    /// drift detection from a state where fast ≈ slow.
    pub const fn seed_cusum_slow_from_baselines(&mut self) {
        self.novelty_bl.seed_cusum_slow_from_fast();
        self.displacement_bl.seed_cusum_slow_from_fast();
        self.surprise_bl.seed_cusum_slow_from_fast();
        self.coherence_bl.seed_cusum_slow_from_fast();
    }

    /// Current maturity snapshot.
    pub const fn maturity(&self) -> TrackerMaturity {
        TrackerMaturity {
            real_observations: self.real_observations,
            noise_observations: self.noise_observations,
            noise_influence: self.noise_influence,
        }
    }

    /// Current rank.
    pub const fn rank(&self) -> usize {
        self.rank
    }

    /// Maximum rank this tracker can reach (`min(dim, max_rank)`).
    pub const fn cap(&self) -> usize {
        self.cap
    }

    /// Working dimensionality of the tracker's input space.
    pub const fn dim(&self) -> usize {
        self.dim
    }

    /// Snapshot the current per-axis baseline means and variances.
    ///
    /// Used for coordination warm-up synthetic score generation
    /// (§ALGO S-9.8).
    pub const fn axis_baselines(&self) -> super::AxisBaselines {
        super::AxisBaselines {
            novelty_mean: self.novelty_bl.fast.mean(),
            novelty_var: self.novelty_bl.fast.variance(),
            displacement_mean: self.displacement_bl.fast.mean(),
            displacement_var: self.displacement_bl.fast.variance(),
            surprise_mean: self.surprise_bl.fast.mean(),
            surprise_var: self.surprise_bl.fast.variance(),
            coherence_mean: self.coherence_bl.fast.mean(),
            coherence_var: self.coherence_bl.fast.variance(),
        }
    }

    /// Geometric properties of the current scoring state.
    pub const fn scoring_geometry(&self) -> ScoringGeometry {
        ScoringGeometry {
            dim: self.dim,
            cap: self.cap,
            residual_dof: self.dim.saturating_sub(self.rank),
        }
    }

    /// Per-axis clip-pressure EWMA values [novelty, displacement, surprise, coherence].
    pub const fn clip_pressures(&self) -> [f64; 4] {
        [
            self.novelty_bl.clip_pressure,
            self.displacement_bl.clip_pressure,
            self.surprise_bl.clip_pressure,
            self.coherence_bl.clip_pressure,
        ]
    }

    // ════════════════════════════════════════════════════════
    //  Private implementation
    // ════════════════════════════════════════════════════════

    /// Build a `faer::Mat<f64>` (b × d) from row slices.
    fn build_matrix(rows: &[&[f64]], b: usize, d: usize) -> Mat<f64> {
        let mut x = Mat::zeros(b, d);
        for (i, row) in rows.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                x[(i, j)] = val;
            }
        }
        x
    }

    /// Phase 1: Compute raw per-sample scores from the prior-model projection.
    ///
    /// Returns the four score vectors. Per-sample `SampleScore` structs
    /// (which include z-scores) are only built when `per_sample_scores`
    /// is enabled — avoiding 4 z-score calls per sample in the common case.
    fn compute_scores(
        &self,
        z: &Mat<f64>,
        residual: &Mat<f64>,
        b: usize,
        d: usize,
        k: usize,
        eps: f64,
    ) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
        #[allow(clippy::cast_precision_loss)] // d − k ≤ 128, well within f64 mantissa
        let dof = (d - k).max(1) as f64;

        #[allow(clippy::cast_precision_loss)]
        let k_f = k as f64;

        let mut nov_scores = Vec::with_capacity(b);
        let mut disp_scores = Vec::with_capacity(b);
        let mut surp_scores = Vec::with_capacity(b);
        let mut coh_scores = Vec::with_capacity(b);

        for i in 0..b {
            // ── Novelty: ‖rᵢ‖² / (d − k) ──
            let mut resid_sq = 0.0;
            for j in 0..d {
                resid_sq = residual[(i, j)].mul_add(residual[(i, j)], resid_sq);
            }
            nov_scores.push(resid_sq / dof);

            // ── Displacement: ‖zᵢ‖² / (k + ‖zᵢ‖²) ──
            let mut z_sq = 0.0;
            for j in 0..k {
                z_sq = z[(i, j)].mul_add(z[(i, j)], z_sq);
            }
            disp_scores.push(z_sq / (k_f + z_sq));

            // ── Surprise: (1/k) Σⱼ (zᵢⱼ − μⱼ)² / (νⱼ + ε) ──
            let mut surprise = 0.0;
            for j in 0..k {
                let dev = z[(i, j)] - self.lat_mean[j];
                surprise += (dev * dev) / (self.lat_var[j] + eps);
            }
            surp_scores.push(surprise / k_f.max(1.0));

            // ── Coherence: (2/(k(k−1))) Σⱼ<ₗ (zᵢⱼ·zᵢₗ − Cⱼₗ)² ──
            //
            // Dividing by pairs = k(k−1)/2 is equivalent to multiplying
            // by 2/(k(k−1)), matching the spec (§6.4).
            coh_scores.push(if k >= 2 {
                let pairs = (k * (k - 1)) / 2;
                let mut coh = 0.0;
                for j in 0..k {
                    for l in (j + 1)..k {
                        let prod = z[(i, j)] * z[(i, l)];
                        let dev = prod - self.cross_corr[tri_idx(j, l, self.cap)];
                        coh = dev.mul_add(dev, coh);
                    }
                }

                #[allow(clippy::cast_precision_loss)]
                {
                    coh / pairs as f64
                }
            } else {
                0.0
            });
        }

        (nov_scores, disp_scores, surp_scores, coh_scores)
    }

    /// Build per-sample `SampleScore` structs (only when `per_sample_scores` is enabled).
    ///
    /// This is separated from `compute_scores` to avoid 4 z-score calls
    /// per sample in the default (disabled) case.
    fn build_per_sample(&self, nov: &[f64], disp: &[f64], surp: &[f64], coh: &[f64], eps: f64) -> Vec<SampleScore> {
        nov.iter()
            .zip(disp)
            .zip(surp)
            .zip(coh)
            .map(|(((&n, &d), &s), &c)| SampleScore {
                novelty: n,
                displacement: d,
                surprise: s,
                coherence: c,
                novelty_z: self.novelty_bl.fast.z_score(n, eps),
                displacement_z: self.displacement_bl.fast.z_score(d, eps),
                surprise_z: self.surprise_bl.fast.z_score(s, eps),
                coherence_z: self.coherence_bl.fast.z_score(c, eps),
            })
            .collect()
    }

    /// Phase 2: Evolve subspace via the maths module (ADR-S-016).
    ///
    /// Delegates to `crate::maths::evolve()` which dispatches to the
    /// selected strategy (naïve dense SVD or Brand's incremental SVD).
    /// In debug builds, both run and are compared.
    ///
    /// Accepts `z` and `residual` from Phase 1 so Brand's method can
    /// reuse the projection instead of recomputing it.
    fn evolve_subspace(&mut self, z: &Mat<f64>, residual: &Mat<f64>, k: usize) {
        let _span = tracing::debug_span!("phase2_evolve_subspace",
            strategy = ?self.svd_strategy,
            k = k,
            d = self.dim,
            b = residual.nrows(),
        )
        .entered();

        let sqrt_lam = self.forgetting_factor.sqrt();

        let Some(update) = crate::maths::evolve(
            self.svd_strategy,
            &self.basis,
            &self.sigmas,
            z,
            residual,
            sqrt_lam,
            k,
            self.cap,
        ) else {
            tracing::warn!("SVD did not converge — keeping prior subspace");
            return;
        };

        // Write back into tracker state.
        let n = update.n;
        for j in 0..n {
            for i in 0..self.dim {
                self.basis[(i, j)] = update.basis[(i, j)];
            }
            self.sigmas[j] = update.sigmas[j];
        }

        // Zero out unused sigmas.
        for s in &mut self.sigmas[n..] {
            *s = 0.0;
        }
    }

    /// Phase 3: Evolve latent distribution (mean, variance, cross-correlation).
    ///
    /// Uses the `Z` matrix from Phase 1 (prior-basis projection).
    ///
    /// **Cold→warm initialisation (§ALGO S-4.2 Phase 3, ADR-S-013 §2a–2c):**
    /// On the first batch (`step == 0`), latent mean, variance, and
    /// cross-correlation are seeded directly from the batch statistics
    /// rather than EWMA-blending against the placeholders (`lat_mean = 0`,
    /// `lat_var = 1.0`, `cross_corr = 0`).  This eliminates the
    /// deterministic cold-start cascade that otherwise inflates surprise
    /// and coherence scores for ~60 rounds.
    fn evolve_latent(&mut self, z: &Mat<f64>, k: usize) {
        let b = z.nrows();
        let lam = self.forgetting_factor;
        let alpha = 1.0 - lam;
        let eps = self.eps;
        let cold = self.step == 0;

        #[allow(clippy::cast_precision_loss)]
        let b_f = b as f64;

        // Per-dimension mean and variance.
        for j in 0..k {
            let mut col_sum = 0.0;
            for i in 0..b {
                col_sum += z[(i, j)];
            }
            let col_mean = col_sum / b_f;

            // EWMA-mean-centred variance (ADR-S-021 §1–2): centre on
            // the pre-update EWMA mean, not the batch mean.  At t = 0
            // lat_mean[j] is 0.0, giving the first-batch seeding formula.
            let mut col_var = 0.0;
            for i in 0..b {
                let d = z[(i, j)] - self.lat_mean[j];
                col_var = d.mul_add(d, col_var);
            }
            col_var /= b_f;

            // Update order (ADR-S-021 §3): variance first (against
            // pre-update mean), then mean.
            if cold {
                // Cold→warm: seed directly from first batch (ADR-S-013 §2a).
                self.lat_var[j] = col_var.max(eps);
                self.lat_mean[j] = col_mean;
            } else {
                self.lat_var[j] = lam.mul_add(self.lat_var[j], alpha * col_var.max(eps));
                self.lat_mean[j] = lam.mul_add(self.lat_mean[j], alpha * col_mean);
            }
        }

        // Runtime floor (ADR-S-021 §4, §ALGO S-4.2): defence-in-depth
        // against degenerate streams.  Caps per-dimension surprise at 100.
        for lat_var in self.lat_var.iter_mut().take(k) {
            *lat_var = (*lat_var).max(1e-2);
        }

        // Pairwise cross-correlation: C[j][l] ← λ·C[j][l] + α·(1/b)·Σᵢ zᵢⱼ·zᵢₗ
        //
        // Invariant (§4.2 Phase 3): when rank increases, new entries are
        // already zero from construction.  When rank decreases, outer
        // entries are ignored here but preserved in the vector.
        for j in 0..k {
            for l in (j + 1)..k {
                let mut prod_sum = 0.0;
                for i in 0..b {
                    prod_sum = z[(i, j)].mul_add(z[(i, l)], prod_sum);
                }
                let batch_corr = prod_sum / b_f;
                let idx = tri_idx(j, l, self.cap);
                if cold {
                    // Cold→warm: seed directly from first batch (ADR-S-013 §2b).
                    self.cross_corr[idx] = batch_corr;
                } else {
                    self.cross_corr[idx] = lam.mul_add(self.cross_corr[idx], alpha * batch_corr);
                }
            }
        }
    }

    /// Phase 4 helper: score against an axis's baseline and (optionally)
    /// evolve it.
    ///
    /// When `evolve` is `false` the EWMA and CUSUM are left untouched.
    /// This is used for the coherence axis at rank < 2, where coherence
    /// does not exist (no pairs) and every score is identically zero.
    /// The baseline stays cold; when rank reaches 2 the first real
    /// values enter through the cold→warm path naturally.  If rank
    /// later drops below 2, `adapt_rank()` destroys the baseline
    /// entirely.
    ///
    /// §ALGO S-6.1.1 pipeline with clip-pressure EWMA (§ALGO S-6.4).
    #[allow(clippy::too_many_arguments)]
    fn update_axis(
        bl: &mut AxisBaseline,
        scores: &[f64],
        eps: f64,
        cusum_allowance: f64,
        clip_sigmas: f64,
        eta: f64,
        clip_pressure_decay: f64,
        evolve: bool,
    ) -> ScoreDistribution {
        // ── Raw batch statistics (pre-clip) ─────────────
        let (min, max, sum) = scores
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY, 0.0_f64), |(mn, mx, s), &v| {
                (mn.min(v), mx.max(v), s + v)
            });

        #[allow(clippy::cast_precision_loss)]
        let mean = sum / scores.len() as f64;

        // Z-scores computed *before* updating the fast baseline.
        let max_z = bl.fast.z_score(max, eps);
        let mean_z = bl.fast.z_score(mean, eps);
        let baseline = bl.fast.snapshot();

        if evolve {
            // ── Per-axis effective clip (§ALGO S-6.4) ───
            //
            //   p = max(η, ρ̄)
            //   n_σ_eff = n_σ · (1 + p / (1 − p + ε))
            //
            let p = eta.max(bl.clip_pressure);
            let effective_clip = clip_sigmas * (1.0 + p / (1.0 - p + eps));

            // ── Single shared clip filter ───────────────
            // Computed against the fast EWMA's current baseline.
            // Cold-path bypass: ceiling() returns +∞ when cold.
            let ceiling = bl.fast.ceiling(effective_clip);

            let retained: Vec<f64> = scores.iter().copied().filter(|&v| v < ceiling).collect();

            // ── Update clip-pressure EWMA ───────────────
            //   ρ_t = 1 − |retained| / |total|
            //   ρ̄ = λ_ρ · ρ̄ + (1 − λ_ρ) · ρ_t
            #[allow(clippy::cast_precision_loss)]
            let rho_t = 1.0 - (retained.len() as f64 / scores.len() as f64);
            let alpha = 1.0 - clip_pressure_decay;
            bl.clip_pressure = clip_pressure_decay.mul_add(bl.clip_pressure, alpha * rho_t);

            // ── Fast EWMA: receives retained samples ────
            if retained.is_empty() {
                // All outliers — learn nothing this round.
                // clip_pressure was still updated above (it saw 100% clipping).
            } else {
                bl.fast.update_raw(&retained);
            }

            // ── CUSUM: receives retained samples, raw batch mean ──
            // Always called — the gap uses raw_batch_mean so CUSUM
            // accumulates even when all scores are clipped.  The slow
            // EWMA's update_raw() is a no-op on empty input.
            bl.cusum.update_filtered(&retained, mean, cusum_allowance, eps);
        }
        let cusum = bl.cusum.snapshot();

        ScoreDistribution {
            min,
            max,
            mean,
            max_z_score: max_z,
            mean_z_score: mean_z,
            baseline,
            cusum,
            clip_pressure: bl.clip_pressure,
        }
    }

    /// Phase 5: Adapt rank based on cumulative energy.
    ///
    /// Every `rank_update_interval` steps, find the smallest rank
    /// capturing `energy_threshold` of total variance.  Move by ±1.
    fn adapt_rank(&mut self) {
        let total_energy: f64 = self.sigmas.iter().map(|s| s * s).sum::<f64>() + self.eps;

        let mut cumulative = 0.0;
        let mut target = self.cap;

        for (i, s) in self.sigmas.iter().enumerate() {
            cumulative += s * s;
            if cumulative / total_energy >= self.energy_threshold {
                target = (i + 2).min(self.cap);
                break;
            }
        }
        target = target.max(1);

        let old_rank = self.rank;

        // Move by at most ±1 to avoid oscillation (§4.2 Phase 5).
        if target > self.rank {
            self.rank = (self.rank + 1).min(self.cap);
        } else if target < self.rank {
            self.rank = self.rank.saturating_sub(1).max(1);
        }

        // Coherence does not exist at rank < 2.  If rank just
        // dropped below 2, destroy the coherence baseline so
        // stale state from a previous k ≥ 2 epoch cannot leak
        // into a future one.  When rank reaches 2 again the
        // baseline is born fresh via the cold→warm path.
        if old_rank >= 2 && self.rank < 2 {
            self.coherence_bl.reset_cold();
        }
    }

    /// Fraction of total variance captured by the current rank.
    pub fn energy_ratio(&self) -> f64 {
        let total: f64 = self.sigmas.iter().map(|s| s * s).sum::<f64>() + self.eps;
        let active: f64 = self.sigmas[..self.rank].iter().map(|s| s * s).sum();
        active / total
    }

    /// Largest singular value of the learned subspace.
    pub fn top_singular_value(&self) -> f64 {
        self.sigmas.first().copied().unwrap_or(0.0)
    }

    /// EWMA latent-coordinate mean for the first `rank` dimensions.
    ///
    /// Exposed for convergence investigation tests (ADR-S-013).
    #[cfg(test)]
    pub fn latent_mean(&self) -> &[f64] {
        &self.lat_mean[..self.rank]
    }

    /// EWMA latent-coordinate variance for the first `rank` dimensions.
    ///
    /// Exposed for convergence investigation tests (ADR-S-013).
    #[cfg(test)]
    pub fn latent_var(&self) -> &[f64] {
        &self.lat_var[..self.rank]
    }

    /// Update maturity counters after processing a batch.
    ///
    /// Noise influence decays as λⁿ for `n` real observations, or
    /// converges toward 1.0 under noise (§11.5). Computed via `powi`
    /// instead of an `n`-iteration loop.
    ///
    /// When η crosses below `WARMUP_THRESHOLD` (§ALGO S-11.4), all
    /// per-axis clip-pressure EWMAs are zeroed to prevent warm-up
    /// contamination from echoing into production scoring.
    fn update_maturity(&mut self, batch_size: usize, is_noise: bool) {
        /// η threshold below which warm-up is considered complete (§ALGO S-11.4).
        const WARMUP_THRESHOLD: f64 = 0.01;

        let count = batch_size as u64;

        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)] // batch_size ≪ 2^31
        let n = batch_size as i32;
        let lam_n = self.forgetting_factor.powi(n);

        let old_eta = self.noise_influence;

        if is_noise {
            self.noise_observations += count;
            // η_{t+n} = λⁿ·η_t + (1 − λⁿ)  (geometric series of n EWMA steps toward 1.0)
            self.noise_influence = lam_n.mul_add(self.noise_influence, 1.0 - lam_n);
        } else {
            self.real_observations += count;
            // η_{t+n} = λⁿ·η_t
            self.noise_influence *= lam_n;
        }

        // §ALGO S-11.4: when η crosses the warm-up threshold, zero
        // clip-pressure to prevent warm-up contamination echoing into
        // production scoring.
        if old_eta >= WARMUP_THRESHOLD && self.noise_influence < WARMUP_THRESHOLD {
            self.novelty_bl.clip_pressure = 0.0;
            self.displacement_bl.clip_pressure = 0.0;
            self.surprise_bl.clip_pressure = 0.0;
            self.coherence_bl.clip_pressure = 0.0;
        }
    }
}

// ─── Upper-triangle indexing ─────────────────────────────────

/// Map a pair `(j, l)` with `j < l` to a flat upper-triangle index.
///
/// The triangle for a `cap × cap` matrix stores `cap*(cap−1)/2`
/// elements in row-major order:  (0,1), (0,2), …, (0,cap−1),
/// (1,2), …, (cap−2, cap−1).
#[inline]
const fn tri_idx(j: usize, l: usize, cap: usize) -> usize {
    // Elements before row j: j*cap − j*(j+1)/2
    // Offset within row j:   l − j − 1
    j * cap - (j * (j + 1)) / 2 + l - j - 1
}
