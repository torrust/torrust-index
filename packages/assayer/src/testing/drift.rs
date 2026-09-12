// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Drift budgets for comparisons whose two sides should agree exactly.
//!
//! These once attributed their spread to `PersistentTimestamp::now()` reads and
//! promised bit-identity once a clock was injected. That attribution was wrong,
//! and measurement settled it: the reported-Sentinel cells were diverging on
//! report staleness, which is read in the *monotonic* domain, and no unreported
//! cell ever failed. Staleness is deterministic now that both domains come from
//! the injected clock (´entry:assayer:harness-stage-clock-monotonic´).
//!
//! What remained after that was a floating-point residue at the scale of a few
//! units in the last place, and beside it a coarser residue that did not vary
//! with elapsed time. The coarser one had two causes and they were told apart by
//! measurement (´entry:assayer:harness-stage-clock-residual-race´). One was the
//! harness's — registrations that returned before the model owner had published
//! them — and it is fixed. The other was read as the identity maintenance thread
//! installing competitive-set changes between the requests of one batch
//! (´entry:assayer:mid-batch-identity-movement´), and the widest budgets here
//! were sized for that reading.
//!
//! Neither cause is live in this suite now, and the second reading did not
//! survive measurement. Nothing here compares requests across a batch any more:
//! a cross-channel comparison holds one assessment and derives every channel
//! from it, so there is no window between requests for anything to move through.
//! What went on failing under load was the pair of cross-world matrix witnesses,
//! and their cause is the standardisation ramp rather than the maintenance
//! thread: the ramp advances on accepted observations, the steward applies them
//! asynchronously, and two worlds compared while still transitioning stand at
//! different accepted counts for no reason but how the threads were scheduled.
//! Those witnesses settle both of their worlds first now, as the paired trainers
//! beside them always have.
//!
//! The cross-world budgets held open for the retired reading are gone with it,
//! measured rather than argued away: run loaded — sixteen concurrent instances
//! of the whole `multi_channel` binary at sixteen test threads on a 112-core
//! host, the same shape that reproduces the flake — every cross-world witness
//! holds at the floating-point scale its narrow sibling uses.
//!
//! The two request-order budgets that outlived that retirement have now been
//! measured on the same shape, and they are retired with it. The window their
//! gloss named — a publication landing between two separate calls — is not on
//! the path: both orders derive from one held assessment, and the
//! batch-versus-singleton witness that used the wider of the two is handed three
//! worlds whose ramps already stand at the horizon. Zeroed first to the
//! floating-point floor the narrow siblings use and then to the purity floor a
//! decade below that, both arms held green in every instance, against a control
//! at the shipped widths that was green as well. Nothing survives between the
//! purity budget and the narrow siblings, so the selector that chose among the
//! three widths went with the constants and the request-order sites name
//! `PURITY_DRIFT` directly. Each budget below says which shape it answers.
//!
//! All of that was measured in the debug profile, and the widths it produced
//! are too narrow for the residue a release build shows. That residue is the
//! optimiser's: the same sums, associated differently because the compiler is
//! free to regroup floating-point work it has been given no reason to keep in
//! order, and the grouping it picks is a property of the whole compilation
//! rather than of the code under comparison. A tag-profile comparison that had
//! held at the purity width in a release container went on to miss it by
//! 9.2e-8 in a build whose only difference was a documentation change
//! elsewhere: the code being compared was identical, and the association had
//! moved because unrelated code had. A width measured under one profile alone
//! is therefore not a width at all — it is a reading of one association among
//! the many a release build may choose — so the budgets below are set to cover
//! the family rather than the draw.
//!
//! The margin is an order of magnitude over the one release residue that has
//! been measured, which still leaves these budgets four decades below the
//! smallest difference they exist to catch. What they hold is that two
//! derivations of one assessment agree: that a live outcome axis does not
//! reach the tags, that reversing a request order does not change an answer.
//! A dependence of that kind is a first-order effect on a quantity of order
//! one — it moves a tag by a fraction of the tag, not by parts in a million.
//! Everything between the measured residue and that scale is the compiler's
//! arithmetic, and refusing it costs the suite its determinism on the profile
//! the release container gates with while catching nothing.

/// Drift budget for "identical inputs, identical answer" comparisons,
/// request-order replays among them.
///
/// Floating-point residue: the engine sums in an order that is not fixed across
/// runs, summation is not associative, and the optimiser regroups those sums
/// differently from one build to the next. Reversing a batch adds nothing to
/// that, because both orders derive from one held assessment.
///
/// The width answers the build rather than the run. A release container
/// measured 9.2e-8 on a tag-profile comparison whose two sides were compiled
/// from identical code, and this is an order of magnitude above that.
pub const PURITY_DRIFT: f64 = 1e-6;

/// Drift budget for public lifecycle-sufficiency witnesses.
///
/// Floating-point residue, one decade wider than [`PURITY_DRIFT`] because the
/// compared statistic is reached through a longer chain of sums — which is
/// also more work for an optimiser to regroup, so the relation holds under the
/// release profile for the reason it held under the debug one.
pub const LIFECYCLE_SUFFICIENCY_DRIFT: f64 = 1e-5;

/// Drift budget for cross-world divergent-outcome replay witnesses.
///
/// Floating-point residue: resonance `q` is a nonlinear transform of the
/// compared statistic, so a last-place difference in the statistic lands a
/// little wider in the public profile. One decade over [`PURITY_DRIFT`]: the
/// transform's amplification is a property of the arithmetic rather than of
/// the build that performs it, so the relation between the two carries across
/// profiles unchanged.
pub const DIVERGENT_OUTCOME_REPLAY_DRIFT: f64 = 1e-5;
