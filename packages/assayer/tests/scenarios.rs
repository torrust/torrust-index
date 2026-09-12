// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`zenos_paradox_halves_forever`] | scenario | The half-life reported for a decay rate is the real thing and not an approximation of it: multiplying together ten decay factors, each taken over one reported half-life, lands on one part in 1024. Zeno's Achilles halves the remaining gap ten times and arrives exactly where arithmetic says. A host reasoning in half-lives is therefore reasoning about the same quantity the engine decays by. |
//! | [`cats_nine_lives`] | scenario | cites (´claim:scenario:the-reported-half-life-is-the-duration-that-exactly-halves-the-decay-factor´) |
//! | [`fibonacci_duration_composition`] | scenario | How a stretch of time is chopped up does not change how much decay it carries: ten unevenly-sized Fibonacci intervals decayed one after another agree with a single decay over their total. Because the factor is exponential in the duration, an engine that updates on ragged, irregular arrivals reaches the same state as one updated on a tidy schedule — the answer cannot be nudged by how often a host happens to call in. |
//! | [`time_travellers_regret_cannot_decay_past`] | scenario | Time running backwards decays nothing rather than un-decaying: a pair of timestamps whose "new" reading precedes its "old" one yields a factor of exactly one. Clock skew, a replayed record or an out-of-order arrival can therefore cost the engine an update but can never amplify stored evidence, which an unclamped exponent over a negative duration would do. |
//! | [`hourglass_inverts_decay`] | scenario | Asking how long it takes to decay to a given fraction and then decaying for that long returns the fraction asked for, across a hundred random targets spread over the interval and to within a part in a trillion. The two directions are a genuine inverse pair, so a host can specify retention either as a duration or as a remaining weight and get the same policy. |
//! | [`half_life_specification_folklore`] | scenario | The decay rates the specification publishes mean in days what it says they mean: roughly a fortnight, a month and most of a year as the rate is pushed from 0.998 through 0.999 to 0.9999. Each extra nine is roughly a tenfold lengthening of memory, which is what makes the rate choosable by an operator thinking in calendar time rather than in exponents. |
//! | [`sigmoid_and_logit_are_mirror_twins`] | scenario | The numerically-stable primitives are true inverses, not merely stable: pushing a thousand random probabilities out to log-odds and back recovers each to within a part in a trillion, right down to a millionth from either boundary where the naive forms lose their footing. Risk moves between probability space and score space repeatedly inside the engine, so any round-trip error would compound rather than cancel. |
//! | [`atanh_undoes_tanh_everywhere`] | scenario | cites (´claim:scenario:the-stable-transforms-recover-their-inputs-across-the-sampled-domain´) |
//! | [`regime_transition_is_monotone_and_bounded`] | scenario | The blend between regimes is a smooth ramp rather than a switch: over a dense grid running from below zero to one it never decreases, never leaves the unit interval, and passes through a half exactly at a weight of 0.3. Monotonicity is what stops a marginal shift in weight from swinging the blend the wrong way, and the boundedness is what lets its output be used directly as a mixing proportion. |
//! | [`goldilocks_scalar_clips_porridge`] | scenario | A scalar signal's declared range is enforced by saturation, not rejection: a value far below the floor encodes as the floor, one far above the ceiling as the ceiling, and anything inside passes through untouched. Clipping at the edge keeps an out-of-range reading from dominating a standardised feature vector while still recording which way it was out. |
//! | [`marathon_runner_stays_bounded`] | scenario | The log-scaled shape needs no declared ceiling to stay usable: inputs climbing from ten to a quintillion each encode to a single finite, strictly positive feature. This is the shape for quantities with no natural upper bound — counts, ages, byte totals — where a host cannot honestly name a clip range in advance and would otherwise have to guess one. |
//! | [`sundial_is_perpendicular_at_quarter_periods`] | scenario | A cyclic signal encodes to a point on the unit circle: the four quarter marks of a twenty-four-hour period all have unit norm, adjacent quarters are orthogonal, and opposite quarters are antipodal. Geometry carries the wrap-around that a raw number cannot — midnight and the hour before it come out neighbours instead of sitting at opposite ends of a scale. |
//! | [`pigeonhole_forces_hash_collision`] | scenario | A hashed categorical always spends exactly one slot of its declared width, whatever string it is given and however many distinct strings it has already seen: ten labels into four buckets each produce a single set position, and the pigeonhole collisions that follow are absorbed rather than resisted. Bounding the width is what lets an open-ended vocabulary be encoded at all, at the price of a few categories sharing a feature. |
//! | [`parrots_perfect_memory`] | scenario | The bucket a category lands in is a function of the string alone: five hundred encodings of one label into a sixty-four-wide space are identical every time. Nothing per-process or per-call seeds the hash, so a feature learned about a category in one run still refers to that category in the next — which a randomised hash would silently break across restarts. |
//! | [`poisoned_apple_sanitised`] | scenario | A non-finite input is neutralised at the encoder rather than carried inward: NaN and both infinities each become zero, and in a vector shape the substitution is per element, so a clean neighbour keeps its own value. One bad reading in a request therefore costs the feature it arrived in and nothing else, instead of turning the whole vector — and every model downstream of it — into NaN. |
//! | [`a_tale_of_two_users_no_crosstalk`] | scenario | Cached signals belong to the entity that supplied them: two entities writing opposite values to the same declared signal, in interleaved order, each read back exactly what they wrote. The cache is keyed by entity rather than by signal name, so one user's history can never be served as another's evidence — the failure that would let a busy account launder its reputation onto a quiet one. |
//! | [`the_forgetful_oracle_keeps_entity_drops_request`] | scenario | The declared persistence of a signal decides what survives a request. An entity-persistent value supplied once is still there on the next request that supplies nothing, while a request-scoped value from the same call has gone back to zero. That split is what lets slow-moving facts about an account carry forward without a per-request measurement — a velocity, a timestamp — being mistaken for a standing property of the entity. |
//! | [`phoenix_rises_after_eviction`] | scenario | Eviction from a full cache is forgetting, not corruption: an entity pushed out by newer arrivals reads back as zero — the same as one never seen — and writing its signal again restores it in full. The cache is therefore a bounded accelerator rather than a store of record, and a host under memory pressure loses recency for cold entities without ever being handed a stale or partial value for them. |
//! | [`russian_dolls_nested_ancestors`] | scenario | The dyadic ancestors of a coordinate nest like dolls: every one of them brackets the coordinate itself, and each deeper ancestor is no wider than the one above it, down to a single point at full depth. A chain of ancestors is thus a genuine refinement of one region rather than a set of unrelated windows, which is what makes evidence gathered at a coarse depth meaningful for anything beneath it. |
//! | [`dyadic_interval_always_a_power_of_two`] | scenario | An ancestor's depth is not a label attached to an arbitrary interval: at every depth from the root to the leaf, including the awkward boundaries at one, sixty-four and a hundred and twenty-seven, the interval produced is recognised as dyadic at that same depth and spans exactly two to the power of the remaining bits. Depth and width are two readings of one fact, so a depth can be compared across coordinates without consulting the interval. |
//! | [`polyglot_signal_schema_speaks_every_shape`] | scenario | Every signal shape coexists in one schema and the layout is exactly the concatenation of their declared widths, in declaration order: seven signals of seven shapes occupy thirteen features, each landing where its predecessors' widths put it, and the entity-persistent mask is expanded to the same layout so a multi-feature signal's slots are all marked together. Position is therefore derivable from the declarations alone, which is what lets a learned weight keep meaning the same thing across processes. |
//! | [`request_context_is_a_builder_not_a_bookkeeper`] | scenario | A request context is a set of facts rather than a log of how they were added: two builds that interleave signals and sentinel coordinates in different orders produce equal entities, equal signals and equal coordinates. Naming the same signal or the same sentinel twice keeps the last value and does not grow the collection. Calling code assembled across several layers can therefore contribute in whatever order it runs, and override an earlier default, without the result depending on the sequence. |

#![allow(clippy::float_cmp, clippy::suboptimal_flops, clippy::imprecise_flops)]

//! Whimsical scenario-based integration tests for `torrust_assayer`.
//!
//! Each test takes a small tale from folklore, mathematics, or physics and
//! turns it into an assertion about the public API. They double as
//! cross-module exercises — every scenario touches at least two modules
//! (numerics + types, signal + types, cache + schema, …).
//!
//! # Cross-References
//!
//! - (´chap:spec:mathematical-foundation´) — the numerics primitives
//! - (´chap:spec:host-signals´) — the signal module
//! - (´chap:spec:assessment-interface´) — core assessment output
//! - (´req:signal:schema-fixed´) — the signal schema fixed at construction, and its cache
//!
//! Uses the shared [`torrust_assayer::testing`] harness for deterministic
//! RNG ([`TestRng`](torrust_assayer::testing::TestRng)), signal-fixture
//! builders ([`signals::scalar_decl`](torrust_assayer::testing::signals::scalar_decl)),
//! and assertion helpers ([`assert_near`](torrust_assayer::testing::assert_near),
//! [`assert_positive`](torrust_assayer::testing::assert_positive),
//! [`assert_in_unit_interval`](torrust_assayer::testing::assert_in_unit_interval),
//! [`assert_finite`](torrust_assayer::testing::assert_finite)).

use std::collections::HashMap;
use std::sync::Arc;

use torrust_assayer::numerics_export::{
    decay_factor, decay_factor_since, half_life_days, half_life_hours, hours_to_decay_target, regime_transition, stable_atanh,
    stable_logit, stable_sigmoid,
};
use torrust_assayer::signal_export::{SignalCache, SignalSchemaIndex, encode_signal};
use torrust_assayer::testing::signals::scalar_decl;
use torrust_assayer::testing::{TestRng, assert_finite, assert_in_unit_interval, assert_near, assert_positive};
use torrust_assayer::types::{EntityKey, PersistentTimestamp, dyadic_ancestor_hi, dyadic_ancestor_lo, is_dyadic};
use torrust_assayer::{Persistence, RequestContext, SentinelId, SignalDeclaration, SignalShape, SignalValue};

fn entity(name: &str) -> EntityKey {
    EntityKey::new(name.as_bytes().to_vec())
}

fn scalar_entity(name: &str) -> SignalDeclaration {
    scalar_decl(name, Persistence::Entity)
}

fn scalar_request(name: &str) -> SignalDeclaration {
    scalar_decl(name, Persistence::Request)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Decay & time folklore
// ═══════════════════════════════════════════════════════════════════════════════

/// The half-life reported for a decay rate is the real thing and not an
/// approximation of it: multiplying together ten decay factors, each taken
/// over one reported half-life, lands on one part in 1024. Zeno's Achilles
/// halves the remaining gap ten times and arrives exactly where arithmetic
/// says. A host reasoning in half-lives is therefore reasoning about the same
/// quantity the engine decays by.
///
/// ´claim:scenario:the-reported-half-life-is-the-duration-that-exactly-halves-the-decay-factor´
/// ´test:integration:zenos-paradox-halves-forever´
#[test]
fn zenos_paradox_halves_forever() {
    let gamma = 0.999;
    let hl = half_life_hours(gamma);

    let mut product = 1.0_f64;
    for _ in 0..10 {
        product *= decay_factor(gamma, hl);
    }

    // 10 half-lives → 1/1024
    let expected = 2_f64.powi(-10);
    assert_near(product, expected, 1e-9, "Achilles after ten halvings (1/1024)");
}

/// Halving does not need to be taken in steps to happen: one contiguous decay
/// over nine half-lives of a different rate arrives at a five-hundred-and-
/// twelfth, to within a part in a trillion. The cat spends its nine lives at
/// once and is exactly as spent as if it had spent them one at a time.
///
/// (´claim:scenario:the-reported-half-life-is-the-duration-that-exactly-halves-the-decay-factor´)
/// ´test:integration:cats-nine-lives´
#[test]
fn cats_nine_lives() {
    let gamma = 0.998;
    let hl = half_life_hours(gamma);

    // Compose nine half-life durations into one contiguous decay.
    let factor = decay_factor(gamma, 9.0 * hl);
    let expected = 2_f64.powi(-9);

    assert_near(factor, expected, 1e-12, "cat after nine lives (1/512)");
}

/// How a stretch of time is chopped up does not change how much decay it
/// carries: ten unevenly-sized Fibonacci intervals decayed one after another
/// agree with a single decay over their total. Because the factor is
/// exponential in the duration, an engine that updates on ragged, irregular
/// arrivals reaches the same state as one updated on a tidy schedule — the
/// answer cannot be nudged by how often a host happens to call in.
///
/// ´claim:scenario:decay-over-a-span-equals-the-product-of-decays-over-its-parts´
/// ´test:integration:fibonacci-duration-composition´
#[test]
fn fibonacci_duration_composition() {
    let gamma = 0.9995;

    // Fib: 1, 1, 2, 3, 5, 8, 13, 21, 34, 55 (sum = 143)
    let fib = [1.0, 1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0, 34.0, 55.0];
    let sum: f64 = fib.iter().sum();

    let product: f64 = fib.iter().map(|&h| decay_factor(gamma, h)).product();
    let direct = decay_factor(gamma, sum);

    assert_near(product, direct, 1e-12, "Fibonacci decay composition");
}

/// Time running backwards decays nothing rather than un-decaying: a pair of
/// timestamps whose "new" reading precedes its "old" one yields a factor of
/// exactly one. Clock skew, a replayed record or an out-of-order arrival can
/// therefore cost the engine an update but can never amplify stored evidence,
/// which an unclamped exponent over a negative duration would do.
///
/// ´claim:scenario:backwards-elapsed-time-decays-nothing-instead-of-amplifying´
/// ´test:integration:time-travellers-regret-cannot-decay-past´
#[test]
fn time_travellers_regret_cannot_decay_past() {
    let past = PersistentTimestamp::new(100_000, 0);
    let present = PersistentTimestamp::new(50_000, 0);

    // We pass "old = past (newer), new = present (older)": going back.
    let factor = decay_factor_since(0.5, &past, &present);
    assert_eq!(factor, 1.0, "Backward time must not decay");
}

/// Asking how long it takes to decay to a given fraction and then decaying for
/// that long returns the fraction asked for, across a hundred random targets
/// spread over the interval and to within a part in a trillion. The two
/// directions are a genuine inverse pair, so a host can specify retention
/// either as a duration or as a remaining weight and get the same policy.
///
/// ´claim:scenario:hours-to-a-decay-target-is-an-exact-inverse-of-the-decay-factor´
/// ´test:integration:hourglass-inverts-decay´
#[test]
fn hourglass_inverts_decay() {
    let gamma = 0.999;
    let mut rng = TestRng::new(0xC0FF_EEBA_BE00_0001_u64);

    for _ in 0..100 {
        // Pick target ∈ (0.01, 1.0]
        let target = 0.01 + 0.99 * rng.next_f64();
        let hours = hours_to_decay_target(gamma, target);
        let round_trip = decay_factor(gamma, hours);
        assert_near(round_trip, target, 1e-12, &format!("hourglass inverse at target {target}"));
    }
}

/// The decay rates the specification publishes mean in days what it says they
/// mean: roughly a fortnight, a month and most of a year as the rate is pushed
/// from 0.998 through 0.999 to 0.9999. Each extra nine is roughly a tenfold
/// lengthening of memory, which is what makes the rate choosable by an
/// operator thinking in calendar time rather than in exponents.
///
/// ´claim:scenario:the-published-decay-rates-carry-their-documented-half-lives-in-days´
/// ´test:integration:half-life-specification-folklore´
#[test]
fn half_life_specification_folklore() {
    assert_near(half_life_days(0.999), 29.0, 1.0, "half-life days for γ=0.999");
    assert_near(half_life_days(0.9999), 290.0, 10.0, "half-life days for γ=0.9999");
    assert_near(half_life_days(0.998), 14.4, 1.0, "half-life days for γ=0.998");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Numerics identities
// ═══════════════════════════════════════════════════════════════════════════════

/// The numerically-stable primitives are true inverses, not merely stable:
/// pushing a thousand random probabilities out to log-odds and back recovers
/// each to within a part in a trillion, right down to a millionth from either
/// boundary where the naive forms lose their footing. Risk moves between
/// probability space and score space repeatedly inside the engine, so any
/// round-trip error would compound rather than cancel.
///
/// ´claim:scenario:the-stable-transforms-recover-their-inputs-across-the-sampled-domain´
/// ´test:integration:sigmoid-and-logit-are-mirror-twins´
#[test]
fn sigmoid_and_logit_are_mirror_twins() {
    let mut rng = TestRng::new(0xDEAD_BEEF_0000_0001_u64);

    for _ in 0..1000 {
        // p ∈ (ε, 1 − ε) to avoid the boundary where logit is infinite
        let eps = 1e-6;
        let p = eps + (1.0 - 2.0 * eps) * rng.next_f64();

        let x = stable_logit(p);
        let back = stable_sigmoid(x);

        assert_near(back, p, 1e-12, &format!("sigmoid(logit({p}))"));
    }
}

/// The stable inverse hyperbolic tangent recovers the argument that the
/// ordinary tangent squashed, across a thousand samples spanning four units
/// either side of the origin. The far ends of that range are where tanh is
/// closest to saturating and a naive inverse would divide by almost nothing,
/// so the round-trip holding to a part in ten billion there is the point.
///
/// (´claim:scenario:the-stable-transforms-recover-their-inputs-across-the-sampled-domain´)
/// ´test:integration:atanh-undoes-tanh-everywhere´
#[test]
fn atanh_undoes_tanh_everywhere() {
    let mut rng = TestRng::new(0xFEED_FACE_0000_0001_u64);

    for _ in 0..1000 {
        // x ∈ (-4, 4)
        let x = 8.0 * rng.next_f64() - 4.0;
        let th = x.tanh();
        let back = stable_atanh(th);
        assert_near(back, x, 1e-10, &format!("atanh(tanh({x}))"));
    }
}

/// The blend between regimes is a smooth ramp rather than a switch: over a
/// dense grid running from below zero to one it never decreases, never leaves
/// the unit interval, and passes through a half exactly at a weight of 0.3.
/// Monotonicity is what stops a marginal shift in weight from swinging the
/// blend the wrong way, and the boundedness is what lets its output be used
/// directly as a mixing proportion.
///
/// ´claim:scenario:the-regime-transition-is-monotone-bounded-and-centred-on-its-declared-weight´
/// ´test:integration:regime-transition-is-monotone-and-bounded´
#[test]
fn regime_transition_is_monotone_and_bounded() {
    let mut prev = regime_transition(-1.0);
    assert_in_unit_interval(prev, "regime_transition(-1.0)");

    for i in -99..=100 {
        let w = f64::from(i) / 100.0;
        let cur = regime_transition(w);
        assert_in_unit_interval(cur, &format!("regime_transition({w})"));
        assert!(cur >= prev - 1e-15, "non-monotone at w={w}: prev={prev}, cur={cur}");
        prev = cur;
    }

    // Crosses 0.5 at w = 0.3
    assert_near(regime_transition(0.3), 0.5, 1e-10, "regime_transition crosses 0.5 at w=0.3");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Signal encoding folklore
// ═══════════════════════════════════════════════════════════════════════════════

/// A scalar signal's declared range is enforced by saturation, not rejection:
/// a value far below the floor encodes as the floor, one far above the ceiling
/// as the ceiling, and anything inside passes through untouched. Clipping at
/// the edge keeps an out-of-range reading from dominating a standardised
/// feature vector while still recording which way it was out.
///
/// ´claim:scenario:a-scalar-signal-saturates-at-its-declared-bounds-and-passes-through-between-them´
/// ´test:integration:goldilocks-scalar-clips-porridge´
#[test]
fn goldilocks_scalar_clips_porridge() {
    let shape = SignalShape::Scalar { clip: (0.0, 100.0) };

    let too_cold = encode_signal(&SignalValue::Numeric(-20.0), &shape);
    let too_hot = encode_signal(&SignalValue::Numeric(999.0), &shape);
    let just_right = encode_signal(&SignalValue::Numeric(42.0), &shape);

    assert_eq!(too_cold, vec![0.0], "too cold should clip to lower bound");
    assert_eq!(too_hot, vec![100.0], "too hot should clip to upper bound");
    assert_eq!(just_right, vec![42.0], "just right should pass through");
}

/// The log-scaled shape needs no declared ceiling to stay usable: inputs
/// climbing from ten to a quintillion each encode to a single finite, strictly
/// positive feature. This is the shape for quantities with no natural upper
/// bound — counts, ages, byte totals — where a host cannot honestly name a
/// clip range in advance and would otherwise have to guess one.
///
/// ´claim:scenario:the-log-scaled-shape-keeps-unbounded-magnitudes-finite-and-positive´
/// ´test:integration:marathon-runner-stays-bounded´
#[test]
fn marathon_runner_stays_bounded() {
    let shape = SignalShape::LogScaled { divisor: 1.0 };

    // Even absurd inputs should be finite (ln grows, but never to infinity
    // for finite input).
    for &miles in &[10.0, 1_000.0, 1_000_000.0, 1e12, 1e18] {
        let encoded = encode_signal(&SignalValue::Numeric(miles), &shape);
        assert_eq!(encoded.len(), 1);
        assert_finite(&encoded, &format!("log-scaled miles={miles}"));
        assert_positive(encoded[0], &format!("log-scaled miles={miles}"));
    }
}

/// A cyclic signal encodes to a point on the unit circle: the four quarter
/// marks of a twenty-four-hour period all have unit norm, adjacent quarters
/// are orthogonal, and opposite quarters are antipodal. Geometry carries the
/// wrap-around that a raw number cannot — midnight and the hour before it come
/// out neighbours instead of sitting at opposite ends of a scale.
///
/// ´claim:scenario:a-cyclic-signal-encodes-to-the-unit-circle-so-wrap-around-is-a-short-distance´
/// ´test:integration:sundial-is-perpendicular-at-quarter-periods´
#[test]
fn sundial_is_perpendicular_at_quarter_periods() {
    let shape = SignalShape::Cyclic { period: 24.0 };

    let q0 = encode_signal(&SignalValue::Numeric(0.0), &shape);
    let q1 = encode_signal(&SignalValue::Numeric(6.0), &shape);
    let q2 = encode_signal(&SignalValue::Numeric(12.0), &shape);
    let q3 = encode_signal(&SignalValue::Numeric(18.0), &shape);

    // Each is a unit vector
    for q in [&q0, &q1, &q2, &q3] {
        let norm = (q[0] * q[0] + q[1] * q[1]).sqrt();
        assert_near(norm, 1.0, 1e-10, &format!("unit norm of {q:?}"));
    }

    // q0 · q1 ≈ 0, q1 · q2 ≈ 0, q2 · q3 ≈ 0 (orthogonal quarters)
    let dot = |a: &[f64], b: &[f64]| a[0] * b[0] + a[1] * b[1];
    assert_near(dot(&q0, &q1), 0.0, 1e-10, "q0 · q1");
    assert_near(dot(&q1, &q2), 0.0, 1e-10, "q1 · q2");
    assert_near(dot(&q2, &q3), 0.0, 1e-10, "q2 · q3");

    // Opposite points are antipodal
    assert_near(dot(&q0, &q2), -1.0, 1e-10, "q0 · q2 antipodal");
    assert_near(dot(&q1, &q3), -1.0, 1e-10, "q1 · q3 antipodal");
}

/// A hashed categorical always spends exactly one slot of its declared width,
/// whatever string it is given and however many distinct strings it has
/// already seen: ten labels into four buckets each produce a single set
/// position, and the pigeonhole collisions that follow are absorbed rather
/// than resisted. Bounding the width is what lets an open-ended vocabulary be
/// encoded at all, at the price of a few categories sharing a feature.
///
/// ´claim:scenario:a-hashed-categorical-spends-exactly-one-slot-and-absorbs-collisions´
/// ´test:integration:pigeonhole-forces-hash-collision´
#[test]
fn pigeonhole_forces_hash_collision() {
    let shape = SignalShape::HashedCategorical { width: 4 };
    let strings = [
        "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa",
    ];

    let mut indices = Vec::with_capacity(strings.len());
    for s in strings {
        let encoded = encode_signal(&SignalValue::Categorical(s.to_string()), &shape);
        assert_eq!(encoded.iter().filter(|&&v| v == 1.0).count(), 1);
        let idx = encoded.iter().position(|&v| v == 1.0).unwrap();
        indices.push(idx);
    }

    // 10 pigeons, 4 holes → collision
    let unique: std::collections::HashSet<_> = indices.iter().copied().collect();
    assert!(
        unique.len() < strings.len(),
        "pigeonhole violated: {} strings, {} unique buckets",
        strings.len(),
        unique.len()
    );
    assert!(unique.len() <= 4, "more unique indices than buckets: {unique:?}");
}

/// The bucket a category lands in is a function of the string alone: five
/// hundred encodings of one label into a sixty-four-wide space are identical
/// every time. Nothing per-process or per-call seeds the hash, so a feature
/// learned about a category in one run still refers to that category in the
/// next — which a randomised hash would silently break across restarts.
///
/// ´claim:scenario:categorical-hashing-is-seeded-by-the-string-alone-so-buckets-are-stable´
/// ´test:integration:parrots-perfect-memory´
#[test]
fn parrots_perfect_memory() {
    let shape = SignalShape::HashedCategorical { width: 64 };
    let sig = SignalValue::Categorical("polly-wants-a-cracker".to_string());

    let first = encode_signal(&sig, &shape);
    for _ in 0..500 {
        assert_eq!(encode_signal(&sig, &shape), first);
    }
}

/// A non-finite input is neutralised at the encoder rather than carried
/// inward: NaN and both infinities each become zero, and in a vector shape the
/// substitution is per element, so a clean neighbour keeps its own value. One
/// bad reading in a request therefore costs the feature it arrived in and
/// nothing else, instead of turning the whole vector — and every model
/// downstream of it — into NaN.
///
/// ´claim:scenario:non-finite-inputs-are-neutralised-per-element-without-touching-clean-neighbours´
/// ´test:integration:poisoned-apple-sanitised´
#[test]
fn poisoned_apple_sanitised() {
    let shape = SignalShape::Scalar { clip: (-10.0, 10.0) };
    assert_eq!(encode_signal(&SignalValue::Numeric(f64::NAN), &shape), vec![0.0]);
    assert_eq!(encode_signal(&SignalValue::Numeric(f64::INFINITY), &shape), vec![0.0]);
    assert_eq!(encode_signal(&SignalValue::Numeric(f64::NEG_INFINITY), &shape), vec![0.0]);

    // Same for Vector shape: per-element sanitisation
    let vshape = SignalShape::Vector {
        len: 3,
        clip: (-1.0, 1.0),
    };
    let encoded = encode_signal(&SignalValue::Vector(vec![f64::NAN, 0.5, f64::INFINITY]), &vshape);
    assert_eq!(encoded.len(), 3);
    assert_eq!(encoded[0], 0.0, "NaN must be sanitised");
    assert_eq!(encoded[1], 0.5, "clean value preserved");
    assert_eq!(encoded[2], 0.0, "infinity must be sanitised");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Cache folklore
// ═══════════════════════════════════════════════════════════════════════════════

/// Cached signals belong to the entity that supplied them: two entities
/// writing opposite values to the same declared signal, in interleaved order,
/// each read back exactly what they wrote. The cache is keyed by entity rather
/// than by signal name, so one user's history can never be served as another's
/// evidence — the failure that would let a busy account launder its reputation
/// onto a quiet one.
///
/// ´claim:scenario:cached-signals-are-keyed-per-entity-so-two-entities-never-cross-talk´
/// ´test:integration:a-tale-of-two-users-no-crosstalk´
#[test]
fn a_tale_of_two_users_no_crosstalk() {
    let schema = Arc::new(SignalSchemaIndex::from_declarations(&[scalar_entity("trust")]).unwrap());
    let cache = SignalCache::new(32, schema);

    let alice = entity("alice");
    let bob = entity("bob");

    let mut alice_signals = HashMap::new();
    alice_signals.insert("trust".to_string(), SignalValue::Numeric(0.9));

    let mut bob_signals = HashMap::new();
    bob_signals.insert("trust".to_string(), SignalValue::Numeric(0.1));

    // Interleave the writes, then apply them: the write is deferred off
    // the merging call, and the drain is where the cache mutates.
    drop(cache.get_and_merge(&alice, &alice_signals));
    drop(cache.get_and_merge(&bob, &bob_signals));
    drop(cache.get_and_merge(&alice, &alice_signals));
    drop(cache.get_and_merge(&bob, &bob_signals));
    cache.drain_deferred_writes();

    // No crosstalk
    assert_eq!(cache.get_and_merge(&alice, &HashMap::new()), vec![0.9]);
    assert_eq!(cache.get_and_merge(&bob, &HashMap::new()), vec![0.1]);
}

/// The declared persistence of a signal decides what survives a request. An
/// entity-persistent value supplied once is still there on the next request
/// that supplies nothing, while a request-scoped value from the same call has
/// gone back to zero. That split is what lets slow-moving facts about an
/// account carry forward without a per-request measurement — a velocity, a
/// timestamp — being mistaken for a standing property of the entity.
///
/// ´claim:scenario:entity-persistent-signals-carry-forward-while-request-scoped-ones-reset´
/// ´test:integration:the-forgetful-oracle-keeps-entity-drops-request´
#[test]
fn the_forgetful_oracle_keeps_entity_drops_request() {
    let schema = Arc::new(
        SignalSchemaIndex::from_declarations(&[scalar_entity("permanent_truth"), scalar_request("fleeting_rumour")]).unwrap(),
    );
    let cache = SignalCache::new(8, schema);
    let who = entity("oracle-client");

    let mut signals = HashMap::new();
    signals.insert("permanent_truth".to_string(), SignalValue::Numeric(0.75));
    signals.insert("fleeting_rumour".to_string(), SignalValue::Numeric(0.99));
    let first = cache.get_and_merge(&who, &signals);
    assert_eq!(first, vec![0.75, 0.99]);
    // The write is deferred off the merging call; the drain applies it.
    cache.drain_deferred_writes();

    // Next request provides nothing: rumour is gone, truth remains.
    let second = cache.get_and_merge(&who, &HashMap::new());
    assert_eq!(second, vec![0.75, 0.0]);
}

/// Eviction from a full cache is forgetting, not corruption: an entity pushed
/// out by newer arrivals reads back as zero — the same as one never seen — and
/// writing its signal again restores it in full. The cache is therefore a
/// bounded accelerator rather than a store of record, and a host under memory
/// pressure loses recency for cold entities without ever being handed a stale
/// or partial value for them.
///
/// ´claim:scenario:an-evicted-entity-reads-as-unseen-and-is-fully-restored-by-writing-again´
/// ´test:integration:phoenix-rises-after-eviction´
#[test]
fn phoenix_rises_after_eviction() {
    let schema = Arc::new(SignalSchemaIndex::from_declarations(&[scalar_entity("flame")]).unwrap());
    let cache = SignalCache::new(2, schema);

    let phoenix = entity("phoenix");
    let mut hot = HashMap::new();
    hot.insert("flame".to_string(), SignalValue::Numeric(1.0));

    // Seed the phoenix; writes are deferred, so drain before reading back.
    drop(cache.get_and_merge(&phoenix, &hot));
    cache.drain_deferred_writes();
    assert_eq!(cache.get_and_merge(&phoenix, &HashMap::new()), vec![1.0]);

    // Overwhelm the cache with other entities. The deferred queue is
    // bounded at the cache's own capacity, so each write is drained as
    // it is made.
    for i in 0..5 {
        let other = entity(&format!("mortal-{i}"));
        let mut sig = HashMap::new();
        sig.insert("flame".to_string(), SignalValue::Numeric(0.1));
        drop(cache.get_and_merge(&other, &sig));
        cache.drain_deferred_writes();
    }

    // The phoenix's ashes: evicted → reads as zero
    assert_eq!(cache.get_and_merge(&phoenix, &HashMap::new()), vec![0.0]);

    // Rise: rewrite and the fire returns
    drop(cache.get_and_merge(&phoenix, &hot));
    cache.drain_deferred_writes();
    assert_eq!(cache.get_and_merge(&phoenix, &HashMap::new()), vec![1.0]);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Dyadic geometry
// ═══════════════════════════════════════════════════════════════════════════════

/// The dyadic ancestors of a coordinate nest like dolls: every one of them
/// brackets the coordinate itself, and each deeper ancestor is no wider than
/// the one above it, down to a single point at full depth. A chain of
/// ancestors is thus a genuine refinement of one region rather than a set of
/// unrelated windows, which is what makes evidence gathered at a coarse depth
/// meaningful for anything beneath it.
///
/// ´claim:scenario:dyadic-ancestors-nest-monotonically-and-narrow-to-a-point-at-full-depth´
/// ´test:integration:russian-dolls-nested-ancestors´
#[test]
fn russian_dolls_nested_ancestors() {
    let coord: u128 = 0xDEAD_BEEF_CAFE_F00D_0123_4567_89AB_CDEF;

    let mut prev_width = u128::MAX;
    for depth in (0_u8..=128).step_by(8) {
        let lo = dyadic_ancestor_lo(coord, depth);
        let hi = dyadic_ancestor_hi(coord, depth);

        // Ancestor contains the coordinate
        assert!(
            lo <= coord && coord <= hi,
            "depth {depth}: [{lo:x}, {hi:x}] does not contain {coord:x}"
        );

        // Width shrinks (or stays equal) as depth increases
        let width = hi - lo;
        assert!(width <= prev_width, "depth {depth}: width {width} > prev {prev_width}");
        prev_width = width;
    }

    // Final doll: a single point
    assert_eq!(dyadic_ancestor_lo(coord, 128), coord);
    assert_eq!(dyadic_ancestor_hi(coord, 128), coord);
}

/// An ancestor's depth is not a label attached to an arbitrary interval: at
/// every depth from the root to the leaf, including the awkward boundaries at
/// one, sixty-four and a hundred and twenty-seven, the interval produced is
/// recognised as dyadic at that same depth and spans exactly two to the power
/// of the remaining bits. Depth and width are two readings of one fact, so a
/// depth can be compared across coordinates without consulting the interval.
///
/// ´claim:scenario:an-ancestor-interval-is-dyadic-at-its-depth-with-width-set-by-the-remaining-bits´
/// ´test:integration:dyadic-interval-always-a-power-of-two´
#[test]
fn dyadic_interval_always_a_power_of_two() {
    let coord: u128 = 0x1234_5678_9ABC_DEF0_1234_5678_9ABC_DEF0;

    for depth in [0_u8, 1, 2, 7, 16, 63, 64, 100, 127, 128] {
        let lo = dyadic_ancestor_lo(coord, depth);
        let hi = dyadic_ancestor_hi(coord, depth);
        assert!(is_dyadic(lo, hi, depth), "ancestor at depth {depth} not dyadic");

        if depth > 0 && depth < 128 {
            let expected_width = 1_u128 << (128 - depth);
            assert_eq!(hi - lo + 1, expected_width, "width wrong at depth {depth}");
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Big-picture scenarios
// ═══════════════════════════════════════════════════════════════════════════════

/// Every signal shape coexists in one schema and the layout is exactly the
/// concatenation of their declared widths, in declaration order: seven signals
/// of seven shapes occupy thirteen features, each landing where its
/// predecessors' widths put it, and the entity-persistent mask is expanded to
/// the same layout so a multi-feature signal's slots are all marked together.
/// Position is therefore derivable from the declarations alone, which is what
/// lets a learned weight keep meaning the same thing across processes.
///
/// ´claim:scenario:a-schema-lays-features-out-as-the-declaration-ordered-concatenation-of-shape-widths´
/// ´test:integration:polyglot-signal-schema-speaks-every-shape´
#[test]
fn polyglot_signal_schema_speaks_every_shape() {
    let decls = vec![
        SignalDeclaration::new("score", SignalShape::Scalar { clip: (0.0, 1.0) }, Persistence::Entity),
        SignalDeclaration::new("age_days", SignalShape::LogScaled { divisor: 1.0 }, Persistence::Request),
        SignalDeclaration::new("verified", SignalShape::Binary, Persistence::Entity),
        SignalDeclaration::new("tier", SignalShape::Ordinal { max: 5.0 }, Persistence::Entity),
        SignalDeclaration::new("hour", SignalShape::Cyclic { period: 24.0 }, Persistence::Request),
        SignalDeclaration::new("country", SignalShape::HashedCategorical { width: 4 }, Persistence::Entity),
        SignalDeclaration::new(
            "embedding",
            SignalShape::Vector {
                len: 3,
                clip: (-1.0, 1.0),
            },
            Persistence::Request,
        ),
    ];

    let schema = SignalSchemaIndex::from_declarations(&decls).unwrap();

    // Widths: 1 + 1 + 1 + 1 + 2 + 4 + 3 = 13
    assert_eq!(schema.total_width(), 13);
    assert_eq!(schema.signal_count(), 7);

    // Entity mask follows the declaration order:
    //   score(E,1) age_days(R,1) verified(E,1) tier(E,1) hour(R,2) country(E,4) embedding(R,3)
    let mask = schema.entity_persistent_mask();
    let expected = vec![
        true,  // score
        false, // age_days
        true,  // verified
        true,  // tier
        false, false, // hour (width 2)
        true, true, true, true, // country (width 4)
        false, false, false, // embedding (width 3)
    ];
    assert_eq!(mask, expected);

    // Encode one of everything at once
    let mut signals = HashMap::new();
    signals.insert("score".to_string(), SignalValue::Numeric(0.5));
    signals.insert("age_days".to_string(), SignalValue::Numeric(0.0)); // ln(1) = 0
    signals.insert("verified".to_string(), SignalValue::Numeric(1.0));
    signals.insert("tier".to_string(), SignalValue::Numeric(5.0)); // clamps to max → 1.0
    signals.insert("hour".to_string(), SignalValue::Numeric(0.0)); // sin(0)=0, cos(0)=1
    signals.insert("country".to_string(), SignalValue::Categorical("FR".to_string()));
    signals.insert("embedding".to_string(), SignalValue::Vector(vec![0.1, -0.2, 0.3]));

    let features = schema.encode_all(&signals);
    assert_eq!(features.len(), 13);

    // Feature ordering mirrors declaration ordering.
    assert_eq!(features[0], 0.5); // score
    assert_eq!(features[1], 0.0); // age_days ln(1+0)=0
    assert_eq!(features[2], 1.0); // verified
    assert_eq!(features[3], 1.0); // tier clamped to max
    assert!(features[4].abs() < 1e-10); // hour sin(0)
    assert!((features[5] - 1.0).abs() < 1e-10); // hour cos(0)
    // features[6..10] = country hashed one-hot — exactly one is 1.0
    let country_slice = &features[6..10];
    assert_eq!(country_slice.iter().filter(|&&v| v == 1.0).count(), 1);
    // features[10..13] = embedding (clipped passthrough)
    assert_near(features[10], 0.1, 1e-12, "embedding[0]");
    assert_near(features[11], -0.2, 1e-12, "embedding[1]");
    assert_near(features[12], 0.3, 1e-12, "embedding[2]");
    // Whole feature vector must be finite
    assert_finite(&features, "polyglot schema features");
}

/// A request context is a set of facts rather than a log of how they were
/// added: two builds that interleave signals and sentinel coordinates in
/// different orders produce equal entities, equal signals and equal
/// coordinates. Naming the same signal or the same sentinel twice keeps the
/// last value and does not grow the collection. Calling code assembled across
/// several layers can therefore contribute in whatever order it runs, and
/// override an earlier default, without the result depending on the sequence.
///
/// ´claim:scenario:a-request-context-is-order-independent-with-last-write-wins-on-repeats´
/// ´test:integration:request-context-is-a-builder-not-a-bookkeeper´
#[test]
fn request_context_is_a_builder_not_a_bookkeeper() {
    let e = EntityKey::new(b"requester".to_vec());

    // Two different build orders producing the same logical result
    let a = RequestContext::new(e.clone())
        .with_signal("velocity", 1.0)
        .with_sentinel(SentinelId(1), 0xAAAA)
        .with_signal("score", 0.5)
        .with_sentinel(SentinelId(2), 0xBBBB);

    let b = RequestContext::new(e)
        .with_sentinel(SentinelId(2), 0xBBBB)
        .with_signal("score", 0.5)
        .with_sentinel(SentinelId(1), 0xAAAA)
        .with_signal("velocity", 1.0);

    assert_eq!(a.entity.as_bytes(), b.entity.as_bytes());
    assert_eq!(a.signals, b.signals);
    assert_eq!(a.sentinel_coordinates, b.sentinel_coordinates);

    // Duplicate signal name: last-write-wins
    let dup = RequestContext::new(EntityKey::new(vec![1]))
        .with_signal("score", 0.1)
        .with_signal("score", 0.9);
    assert_eq!(dup.signals.len(), 1);
    assert_eq!(dup.signals.get("score"), Some(&SignalValue::Numeric(0.9)));

    // Duplicate sentinel id: last-write-wins
    let dup_s = RequestContext::new(EntityKey::new(vec![1]))
        .with_sentinel(SentinelId(42), 0x1111)
        .with_sentinel(SentinelId(42), 0x2222);
    assert_eq!(dup_s.sentinel_coordinates.len(), 1);
    assert_eq!(dup_s.sentinel_coordinates.get(&SentinelId(42)), Some(&0x2222_u128));
}
