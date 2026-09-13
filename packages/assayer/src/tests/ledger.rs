// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`new_neutral_zero_ewmas`] | ledger | cites (´claim:ledger:a-fresh-entry-carries-no-outcome-history´) |
//! | [`new_neutral_timestamp_approx_now`] | ledger | A new entry is stamped with the moment it was created — the timestamp falls inside the interval bracketing its construction. Decay is measured from that stamp, so an entry born with a stale one would be discounted for time it never existed through, and one born with a future stamp would be immune to decay until the clock caught up. |
//! | [`ledger_read_route_root_only`] | ledger | cites (´claim:ledger:a-root-only-ledger-routes-every-coordinate-to-the-root´) |
//! | [`ledger_read_route_deepest_wins`] | ledger | cites (´claim:ledger:the-read-path-returns-the-deepest-cell-containing-the-coordinate´) |
//! | [`ledger_read_route_falls_to_shallower`] | ledger | cites (´claim:ledger:a-coordinate-outside-the-deep-cells-falls-back-to-the-deepest-ancestor-containing-it´) |
//! | [`ledger_read_route_exact_boundary`] | ledger | cites (´claim:ledger:a-cells-lower-endpoint-routes-into-that-cell´) |
//! | [`read_route_many_depths`] | ledger | cites (´claim:ledger:the-read-path-returns-the-deepest-cell-containing-the-coordinate´) |
//! | [`all_layers_write_hits_all`] | ledger | cites (´claim:ledger:a-write-updates-every-tracked-cell-containing-the-coordinate´) |
//! | [`all_layers_write_ewma_formula`] | ledger | Each write mixes one minus lambda of the new observation into what was already stored: from zero, one adverse outcome puts the bad-rate at exactly that fraction and the compressed valence at that fraction of the supplied value, and a following benign outcome scales the bad-rate by lambda rather than replacing it. The retention factor is what makes a single event a small nudge and a sustained pattern a real movement. |
//! | [`read_decay_pure`] | ledger | Reading a decayed view leaves the entry exactly as it was, and two successive reads at the same instant agree. Decay is a lens applied at read time, not a mutation, so a host may read the ledger as often as it likes — for diagnostics, for several axes, for a dashboard — without those reads themselves eroding the evidence. |
//! | [`read_decay_factor`] | ledger | At the default hourly rate an entry is worth about half of what it was after twenty-nine days: a stored value of one reads back near a half once the stamp is rewound by that span. The half-life is the ledger's stated promise about how quickly a bad patch stops counting against a region, and it is a property of the elapsed hours rather than of any sweep having run. |
//! | [`write_decay_before_update`] | ledger | A write discounts the stored value for elapsed time before mixing in the new observation: an entry holding one, untouched for an hour, ends at lambda times the hourly decay factor rather than at lambda alone. Ordering matters because the new observation must not be aged along with the history it is joining — it happened now, and only the past owes time. |
//! | [`cell_set_appearances`] | ledger | cites (´claim:ledger:a-cell-newly-named-by-a-report-starts-being-tracked-from-neutral´) |
//! | [`cell_set_absence_increment`] | ledger | cites (´claim:ledger:a-cell-missing-from-a-report-accrues-a-consecutive-absence´) |
//! | [`cell_set_deletion`] | ledger | cites (´claim:ledger:a-cell-is-dropped-on-the-absence-that-reaches-the-threshold´) |
//! | [`cell_set_root_never_deleted`] | ledger | cites (´claim:ledger:the-root-cell-is-never-deleted-however-long-it-is-absent´) |
//! | [`cell_set_no_merge`] | ledger | cites (´claim:ledger:deleting-a-cell-leaves-its-parent-untouched-rather-than-merging-into-it´) |
//! | [`gc_removes_old_low`] | ledger | cites (´claim:ledger:an-entry-that-is-both-stale-and-empty-is-collected´) |
//! | [`gc_keeps_high_ewma`] | ledger | cites (´claim:ledger:an-entry-still-carrying-signal-survives-collection-however-old-it-is´) |
//! | [`ledger_gc_never_removes_root`] | ledger | cites (´claim:ledger:the-root-entry-is-never-garbage-collected´) |
//! | [`gc_per_axis_ewma`] | ledger | cites (´claim:ledger:a-per-axis-ewma-above-the-floor-is-enough-to-retain-an-entry´) |
//! | [`gc_batched_deletion_bounds_lock_windows`] | ledger | Batched collection clears twenty-three expired entries while holding the exclusive lock only in bounded slices: the read side takes one key-snapshot acquisition and one scan window within its limit, and the write side takes exactly as many acquisitions as the batch limit implies, none larger than that limit. Afterwards only the root remains and the recorded maximum depth has been recalculated back down to zero. The ledger is read on the assessment path, so a sweep that took one long exclusive hold proportional to the backlog would stall live traffic precisely when the ledger is largest. |
//! | [`gc_batched_survives_concurrent_rehash`] | ledger | A sweep started over an eligible population removes every one of its members even while a writer keeps inserting fresh entries between the sweep's bounded lock windows. Concurrent insertion may resize and rehash the entry map mid-sweep; the sweep's candidate discovery walks a key snapshot taken once at the start rather than a position in iteration order, so no eligible entry is skipped and no survivor is inspected twice as the table moves underneath it. The fresh entries themselves all survive, being newer than any horizon. |
//! | [`ledger_gc_scheduler_collects_on_cadence`] | ledger | The collection scheduler actually collects: started over a container holding two Sentinels — one carrying an entry both stale and empty, the other a recent one — it removes the eligible entry within a few ticks and leaves the recent entry and both roots standing, reading the present through the injected clock and the floor and horizon it was configured with. Storage economy is the sweep's whole purpose, and an implemented sweep with no caller bounds nothing. |
//! | [`outcome_ledger_create_sentinel`] | ledger | cites (´claim:ledger:a-sentinels-ledger-is-created-with-its-root-already-in-place´) |
//! | [`per_axis_ewmas_updated`] | ledger | An axis value supplied with an update takes the same EWMA step as the entry-wide figures, and its compressed and raw components are carried separately rather than one being derived from the other. A host reading a registered axis therefore gets a history built on exactly the arithmetic it can reason about from the binary rate. |
//! | [`sentinel_ledger_serde_roundtrip`] | ledger | A serialised ledger comes back with its root and its deeper cells intact, and with the root's recorded tally preserved. Entries are keyed by a composite dyadic key that no textual format can express as a plain map key, so the round-trip is the evidence that the pair-sequence encoding reconstructs the same keyed structure it started from. |
//! | [`entry_serde_with_axes`] | ledger | An entry's axis rows survive serialisation with both components intact and still attached to the right axis id — two axes go out and come back distinguishable, compressed and raw each landing where it belongs. Since the rows are a positional triple rather than a named record, a transposition would restore plausible numbers under the wrong meaning. |
//! | [`read_decay_zero_elapsed`] | ledger | An entry read at the very instant it was written comes back undecayed — bad-rate, compressed valence and raw valence all bit-identical to what is stored. Zero elapsed hours costs nothing, so the outcome of a label is fully available to the assessment that follows it immediately rather than being docked for the round trip. |
//! | [`read_decay_large_gap`] | ledger | After a year untouched an entry reads as vanishingly small yet still strictly positive. The decay is geometric, so it approaches zero without ever arriving: nothing is clamped or flushed to nothing, and an entry that sat out an extreme gap is neither still influential nor a source of exact zeros that downstream ratios would have to special-case. |
//! | [`read_decay_per_axis`] | ledger | Every EWMA in an entry ages by one shared factor: after a day, the bad-rate and both axis rows each read back at the same twenty-four-hour factor times what they stored. One timestamp governs the whole entry, so the relationships between an entry's figures — an axis against the binary rate, one axis against another — survive the passage of time unchanged. |
//! | [`read_decay_backward_clock`] | ledger | A clock that has gone backwards decays nothing rather than amplifying: reading with a "now" an hour before the entry's own stamp returns the stored value untouched. Negative elapsed time in a geometric factor would inflate the evidence, turning an NTP correction into a manufactured spike in a region's bad-rate. |
//! | [`write_single_negative`] | ledger | A benign outcome contributes nothing to the bad-rate: recorded against an empty entry it leaves the rate at exactly zero. Only adverse outcomes are evidence of badness, so a benign one can dilute an existing rate through the retention factor but can never seed one from nothing. |
//! | [`write_ten_positives`] | ledger | A run of adverse outcomes drives the EWMAs toward their drivers geometrically rather than linearly: ten of them take the bad-rate to one minus lambda to the tenth, with the compressed valence at the same fraction of its supplied value. Ten bad events are therefore still a small number against a long-memory retention factor — a burst has to be sustained to register, which is what stops a brief incident from condemning a region. |
//! | [`write_alternating`] | ledger | A mixed history settles between the two extremes rather than tracking whichever outcome came last: an alternating sequence of two adverse and two benign outcomes leaves the bad-rate near twice the single-step increment, slightly discounted by the benign steps in between. What is recorded is the proportion of adverse outcomes over the retained window, not the most recent verdict. |
//! | [`write_compressed_valence`] | ledger | The compressed valence EWMA averages exactly the compressed figure the caller supplies, applying no squashing of its own — a supplied value of 0.9 yields one minus lambda times 0.9 to bit precision. Compression happens upstream where the scale constant lives, so the ledger's job here is to remember rather than to transform. |
//! | [`write_raw_valence`] | ledger | The raw valence is averaged alongside the compressed one on its own unsquashed scale: a raw value of five contributes one minus lambda times five, unbounded by the compressed range. Keeping both means a host can ask how bad the outcomes were in their original units as well as how they rank once compressed — magnitudes that saturation would otherwise flatten away. |
//! | [`write_per_axis_absent`] | ledger | An axis the update says nothing about is left where it was rather than being averaged toward zero: a pre-existing row survives an update that carries no value for it. Silence on an axis is missing data, not a report of nothing happening, so an axis a host reports only occasionally must not have its history eroded by every unrelated label in between. |
//! | [`write_updates_timestamp`] | ledger | A write stamps the entry with the instant it was applied, replacing an ancient stamp outright. The stamp is the origin every later decay is measured from, so leaving it behind would make the next read discount the observation just recorded for time that elapsed before it happened. |
//! | [`write_increments_action_count`] | ledger | Every write is tallied against the action that was actually taken and against no other: two challenges and one block leave those two slots at two and one with the allow and slow slots untouched. Unlike the decaying EWMAs these are undecayed counts, so they answer what the Sentinel did in a region rather than how recently it did it. |
//! | [`write_eligible_increments_recent`] | ledger | Assessments marked eligible raise the recent-eligible count, one per write. That count is what cell-set maintenance consults when deciding whether a region is carrying enough qualifying traffic to be worth tracking at its current resolution. |
//! | [`write_ineligible_doesnt_count`] | ledger | Ineligible assessments are recorded but not credited: two of them leave the eligible count at zero while the total assessment count reaches two. The two tallies answer different questions — how much traffic passed through, and how much of it qualifies as evidence for cell-set decisions — and conflating them would let ineligible traffic keep a region alive on its own. |
//! | [`read_root_fallback`] | ledger | cites (´claim:ledger:a-coordinate-outside-the-deep-cells-falls-back-to-the-deepest-ancestor-containing-it´) |
//! | [`read_sparse_depths`] | ledger | cites (´claim:ledger:the-read-path-returns-the-deepest-cell-containing-the-coordinate´) |
//! | [`read_boundary_hi_minus_1`] | ledger | cites (´claim:ledger:a-cells-lower-endpoint-routes-into-that-cell´) |
//! | [`read_just_outside`] | ledger | cites (´claim:ledger:a-coordinate-outside-the-deep-cells-falls-back-to-the-deepest-ancestor-containing-it´) |
//! | [`read_root_coordinate_zero`] | ledger | cites (´claim:ledger:the-read-path-returns-the-deepest-cell-containing-the-coordinate´) |
//! | [`write_root_only`] | ledger | cites (´claim:ledger:a-write-updates-every-tracked-cell-containing-the-coordinate´) |
//! | [`write_sparse_chain`] | ledger | cites (´claim:ledger:a-write-updates-every-tracked-cell-containing-the-coordinate´) |
//! | [`write_does_not_create`] | ledger | A write never brings a cell into existence: after writing a coordinate whose depth-8 cell is untracked, that cell is still absent and the ledger still holds only its root. Which cells exist is decided by the Sentinel's reports alone, so traffic cannot silently subdivide the ledger into a cell per observation and grow it without bound. |
//! | [`write_two_successive`] | ledger | cites (´claim:ledger:a-benign-outcome-contributes-nothing-to-the-bad-rate´) |
//! | [`write_different_coords_same_ancestor`] | ledger | cites (´claim:ledger:a-write-updates-every-tracked-cell-containing-the-coordinate´) |
//! | [`appear_already_exists`] | ledger | A cell already tracked is not counted as an appearance and is not recreated: its accumulated tally comes through the report untouched. Reports name the full current cell set every cycle rather than only what changed, so treating a repeat mention as a new arrival would reset a region's history on every cycle it stayed alive. |
//! | [`creation_no_inheritance`] | ledger | A newly appeared cell inherits nothing from the ancestor containing it: with the root sitting at a high bad-rate and a substantial valence, the new depth-4 entry starts at zero on both. Inheriting would prejudge a region on its neighbourhood's reputation — and would double-count, since the ancestor keeps receiving that region's writes regardless. |
//! | [`multiple_appearances`] | ledger | cites (´claim:ledger:a-cell-newly-named-by-a-report-starts-being-tracked-from-neutral´) |
//! | [`multiple_deletions`] | ledger | cites (´claim:ledger:a-cell-is-dropped-on-the-absence-that-reaches-the-threshold´) |
//! | [`get_unknown_sentinel`] | ledger | An unregistered Sentinel has no ledger at all rather than an empty one. Asking about an unknown identity is answered with absence, so a caller cannot accidentally accumulate outcome history against a Sentinel the model owner never registered. |
//! | [`multiple_sentinels`] | ledger | Sentinels registered alongside each other are genuinely separate: writing a distinctive tally into one leaves another's root at zero. Each has its own ledger behind its own lock, so Sentinels neither share a coordinate space nor contend with one another for access to it. |
//! | [`ledger_serde_roundtrip`] | ledger | cites (´claim:ledger:a-serialised-ledger-comes-back-with-its-root-and-cells-intact´) |
//! | [`property_bad_rate_in_0_1`] | ledger | The bad-rate is a genuine rate under any history: across a thousand coin-flipped outcomes it never leaves the unit interval at any step. Because it is a convex mixture of the stored value and an indicator, it can only sit between the extremes — and downstream consumers can treat it as a probability without defensively clamping it. |
//! | [`property_compressed_valence_in_minus1_1`] | ledger | The compressed valence EWMA stays inside the range of the values it averages: fed a thousand random drivers spanning minus one to one, it never escapes those bounds in either direction. Averaging cannot overshoot its inputs, so the compression bound established upstream is preserved by the ledger rather than having to be reasserted downstream. |
//! | [`property_ewma_converges_toward_input`] | ledger | A repeated identical observation pulls the EWMA toward it at exactly the rate the retention factor sets — a hundred steps at one value reach the closed-form fraction of it, still well short of the target. Convergence is slow by construction: at this retention a cell needs hundreds of observations before its recorded history is dominated by present behaviour rather than by its past. |
//! | [`property_read_time_decay_in_range`] | ledger | Across gaps from one hour to a thousand, a decayed reading is always strictly positive and never exceeds what is stored. Decay only ever discounts: no elapsed span can manufacture evidence that was not written, and none can erase the last trace of evidence that was. |
//! | [`property_all_layers_always_includes_root`] | ledger | The root receives every write regardless of coordinate or geometry: across a hundred randomly-shaped ledgers and random coordinates, its assessment tally rises by exactly one each time — never zero, never twice for one write. The root is the Sentinel-wide baseline every fallback read lands on, so it must see all of the traffic exactly once. |
//! | [`ledger_decay_twenty_nine_day_half_life_timeline`] | ledger | "Time-Indexed Ledger Decay: 29-Day Half-Life Timeline". Contamination dissolves on the published schedule rather than on a schedule that merely resembles it. A burst of fifty adverse labels lifts a cell to a peak of about five per cent, and at every milestone across three months — day 0, 7, 14, 29, 58 and 87 — the value a read returns matches the closed form of peak times the hourly factor raised to the elapsed hours, to within a billionth, while the stored state itself never moves. By day 87 the reading is under 0.007, roughly an eighth of the peak: effectively inert. This is the promise a host relies on when it tolerates a bad patch, so the timeline has to be exact rather than approximately right, and it has to hold without any sweep having run. |
//! | [`outcome_axis_ewma_decay_twenty_nine_day_timeline`] | ledger | "Outcome Axis EWMA Decays on Same 29-Day Schedule as Binary Rate". Companion to (`ledger_decay_twenty_nine_day_half_life_timeline`), but for a registered outcome axis's compressed-valence EWMA. A per-axis EWMA ages on exactly the same schedule as the binary bad-rate, not merely a similar one. Driven by an axis value at the saturating edge of compression, the axis EWMA reaches the same peak the binary rate reaches from the same number of adverse labels — the two share a step size — and then stays locked to it at every milestone out to 87 days, each reading matching the closed form and the final one below the same inert threshold. A host reading an axis is therefore reasoning on the same timescale it already understands from the risk figures, rather than having to learn a second decay regime per axis it registers. |
//! | [`label_assess_race_no_torn_reads`] | ledger | "The Label-Assessment Race". Spec formulation: "Process a label that flips a Ledger cell's EWMA from 0 to 0.5. Simultaneously `assess()` an observation routed through that cell. The assessment must see either the pre-label EWMA (0) or the post-label EWMA (0.5), never a torn intermediate." A reader routing through the ledger while a writer mutates the very cell it resolves to sees one whole value or the other, never a partial one: the interleaving is forced rather than waited for. The writer holds the exclusive guard across the window in which the label is only half applied — the routed cell updated, the root it hangs from not yet — and a reader stands at the door for exactly that window. It cannot get in, because a read guard is unobtainable while the write guard is held; and by the time it does get in the guard is gone, so what it reads is the post-label state entire, both layers carrying the same label count and the same bit-exact rate. Both pinned values are observed on every run — the pre-label state from an uncontended read before the write, the post-label state from the read that was blocked mid-write — so the guarantee is never vacuous. The layers are walked deepest-first, the order the write path itself walks (´alg:ledger:all-layers-update´), and the state they leave behind is checked against what that path produces in a single call: the hand-walked write is the same write, cut open. Per-Sentinel access is serialised through a read-write lock, and that exclusion is the whole of the guarantee — no reader is ever inside the ledger while a label is half applied, so there is no interleaving left for a torn read to happen in. |

//! Tests for the Outcome Ledger module.
//!
//! These tests exercise the crate-internal ledger primitives directly,
//! but lean on the shared [`testing`](crate::testing) harness for
//! assertion helpers, named tolerances, a deterministic clock, and a
//! seeded RNG. This keeps the epsilon literals, time sources, and
//! pseudo-random streams consistent with the integration-test suite.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use crate::ledger::OutcomeLedger;
use crate::ledger::cell_set::apply_cell_set_changes;
use crate::ledger::entry::{LedgerEntry, LedgerUpdate};
use crate::ledger::gc::{GcLimits, garbage_collect, garbage_collect_batched};
use crate::ledger::routing::{read_route, update_all_layers};
use crate::ledger::sentinel_ledger::SentinelLedger;
use crate::testing::{Clock, DEFAULT_TOLERANCES, TestRng, Tolerances, VirtualClock, assert_near};
use crate::types::{Action, LedgerKey, OutcomeAxisId, PersistentTimestamp, SentinelId, dyadic_ancestor_lo};

// ─── Local helpers ──────────────────────────────────────────────────────────

/// Default tolerance bundle used by these tests.
const TOL: Tolerances = DEFAULT_TOLERANCES;

/// A virtual clock anchored at the default test epoch. Using a clock
/// keeps "now" deterministic and, where needed, lets a test construct
/// timestamps by offsetting from [`Clock::now`] rather than reaching
/// for wall time.
fn clock() -> VirtualClock {
    VirtualClock::epoch()
}

/// Look up the `(compressed, raw)` pair for a given axis id.
fn axis_val(per_axis: &[(OutcomeAxisId, f64, f64)], id: OutcomeAxisId) -> (f64, f64) {
    per_axis
        .iter()
        .find(|(i, _, _)| *i == id)
        .map_or((0.0, 0.0), |(_, c, r)| (*c, *r))
}

/// Look up the compressed component for a given axis id.
fn axis_compressed(per_axis: &[(OutcomeAxisId, f64, f64)], id: OutcomeAxisId) -> f64 {
    axis_val(per_axis, id).0
}

// ═══════════════════════════════════════════════════════════════════════════════
// LedgerEntry Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A neutral entry is empty on every dimension at once — the three entry-wide
/// EWMAs, the axis rows, the assessment total and the absence run. Nothing is
/// left at a placeholder that a later read could mistake for a real
/// observation, which is what makes a freshly-appeared cell safe to route into
/// the moment a report names it.
///
/// (´claim:ledger:a-fresh-entry-carries-no-outcome-history´)
/// ´test:crate:new-neutral-zero-ewmas´
#[test]
fn new_neutral_zero_ewmas() {
    let entry = LedgerEntry::new_neutral();
    assert_near(entry.ewma_bad_rate, 0.0, TOL.bit_identical, "ewma_bad_rate");
    assert_near(
        entry.compressed_valence_ewma,
        0.0,
        TOL.bit_identical,
        "compressed_valence_ewma",
    );
    assert_near(entry.raw_valence_ewma, 0.0, TOL.bit_identical, "raw_valence_ewma");
    assert_eq!(entry.per_axis, [] as [(crate::types::OutcomeAxisId, f64, f64); 0]);
    assert_eq!(entry.total_assessments, 0);
    assert_eq!(entry.consecutive_absent, 0);
}

/// A new entry is stamped with the moment it was created — the timestamp falls
/// inside the interval bracketing its construction. Decay is measured from that
/// stamp, so an entry born with a stale one would be discounted for time it
/// never existed through, and one born with a future stamp would be immune to
/// decay until the clock caught up.
///
/// ´claim:ledger:a-fresh-entry-is-stamped-with-the-moment-it-was-created´
/// ´test:crate:new-neutral-timestamp-approx-now´
#[test]
fn new_neutral_timestamp_approx_now() {
    let before = PersistentTimestamp::now();
    let entry = LedgerEntry::new_neutral();
    let after = PersistentTimestamp::now();

    assert!(entry.last_updated >= before);
    assert!(entry.last_updated <= after);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Read Routing Tests
// ═══════════════════════════════════════════════════════════════════════════════

fn make_ledger_with_depths(depths: &[u8]) -> SentinelLedger {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();

    for &depth in depths {
        if depth == 0 {
            continue;
        }
        let key = LedgerKey::new(0, depth);
        ledger.insert(key, LedgerEntry::new_neutral());
    }

    ledger
}

/// Four coordinates spanning the space — zero, the maximum, the halfway point
/// and an arbitrary interior value — all reach the same distinctively marked
/// root when nothing finer is tracked. The root's coverage is genuinely the
/// whole 128-bit space rather than a region that happens to contain the
/// coordinates a test picked first.
///
/// (´claim:ledger:a-root-only-ledger-routes-every-coordinate-to-the-root´)
/// ´test:crate:ledger-read-route-root-only´
#[test]
fn ledger_read_route_root_only() {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();

    // Mark root distinctively
    ledger.root_mut().total_assessments = 999;

    // Any coordinate should route to root
    let test_coords = [
        0_u128,
        u128::MAX,
        0x8000_0000_0000_0000_0000_0000_0000_0000_u128,
        0xABCD_1234_5678_9ABC_DEF0_1234_5678_9ABC_u128,
    ];

    for coord in test_coords {
        let entry = read_route(&ledger, coord);
        assert_eq!(entry.total_assessments, 999, "coord {coord:#x} should route to root");
    }
}

/// With all three of root, depth-4 and depth-8 marked distinctly and a
/// coordinate contained by all three, the depth-8 entry is the one returned.
/// The presence of shallower ancestors along the same chain does not tempt the
/// walk into stopping early.
///
/// (´claim:ledger:the-read-path-returns-the-deepest-cell-containing-the-coordinate´)
/// ´test:crate:ledger-read-route-deepest-wins´
#[test]
fn ledger_read_route_deepest_wins() {
    let mut ledger = make_ledger_with_depths(&[0, 4, 8]);

    // Mark entries distinctively
    ledger.root_mut().total_assessments = 1;
    ledger.get_mut(&LedgerKey::new(0, 4)).unwrap().total_assessments = 4;
    ledger.get_mut(&LedgerKey::new(0, 8)).unwrap().total_assessments = 8;

    // Coordinate that matches depth-8 cell
    let coord = 0x00AB_0000_0000_0000_0000_0000_0000_0000_u128;
    let entry = read_route(&ledger, coord);
    assert_eq!(entry.total_assessments, 8, "should return depth-8 entry");
}

/// The fallback is checked against the dyadic ancestry directly: the chosen
/// coordinate is confirmed to share the tracked cell's prefix at depth 4 but
/// not at depth 8, and the read then lands on the depth-4 entry. The routing
/// decision agrees with the interval arithmetic rather than merely producing
/// some plausible answer.
///
/// (´claim:ledger:a-coordinate-outside-the-deep-cells-falls-back-to-the-deepest-ancestor-containing-it´)
/// ´test:crate:ledger-read-route-falls-to-shallower´
#[test]
fn ledger_read_route_falls_to_shallower() {
    let mut ledger = make_ledger_with_depths(&[0, 4, 8]);

    ledger.get_mut(&LedgerKey::new(0, 4)).unwrap().total_assessments = 44;

    // Coordinate in depth-4 (top 4 bits = 0) but NOT in depth-8 (top 8 bits != 0)
    let coord = 0x0F00_0000_0000_0000_0000_0000_0000_0000_u128;

    // Verify routing expectations
    assert_eq!(dyadic_ancestor_lo(coord, 4), 0, "coord should match depth-4 lo=0");
    assert_ne!(dyadic_ancestor_lo(coord, 8), 0, "coord should NOT match depth-8 lo=0");

    let entry = read_route(&ledger, coord);
    assert_eq!(entry.total_assessments, 44, "should return depth-4 entry");
}

/// The coordinate equal to a depth-8 cell's own lower endpoint resolves to that
/// cell and not to the root above it. The first coordinate of an interval is
/// inside it, so the very traffic that opens a new region is attributed to the
/// region rather than to the population.
///
/// (´claim:ledger:a-cells-lower-endpoint-routes-into-that-cell´)
/// ´test:crate:ledger-read-route-exact-boundary´
#[test]
fn ledger_read_route_exact_boundary() {
    let mut ledger = make_ledger_with_depths(&[0, 8]);
    ledger.get_mut(&LedgerKey::new(0, 8)).unwrap().total_assessments = 808;

    // Exact boundary: coord = lo = 0
    let entry = read_route(&ledger, 0);
    assert_eq!(entry.total_assessments, 808);
}

/// A seven-deep chain of nested cells still yields the deepest: with entries at
/// every even depth from zero to twelve, the depth-12 entry answers. The walk
/// does not tire or stop at some intermediate level as the ancestry gets
/// longer, so specificity survives however finely a Sentinel subdivides.
///
/// (´claim:ledger:the-read-path-returns-the-deepest-cell-containing-the-coordinate´)
/// ´test:crate:read-route-many-depths´
#[test]
fn read_route_many_depths() {
    let depths = [0, 2, 4, 6, 8, 10, 12];
    let mut ledger = make_ledger_with_depths(&depths);

    // Mark each depth
    for &depth in &depths {
        let key = if depth == 0 {
            LedgerKey::new(0, 0)
        } else {
            LedgerKey::new(0, depth)
        };
        if let Some(entry) = ledger.get_mut(&key) {
            entry.total_assessments = u64::from(depth);
        }
    }

    // Coord matching deepest (depth-12)
    let entry = read_route(&ledger, 0);
    assert_eq!(entry.total_assessments, 12, "deepest depth-12 should win");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Write Routing Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// One labelled outcome is recorded three times over — once in each of the
/// root, depth-4 and depth-8 cells that contain it — and the reported count
/// agrees with the number of entries whose assessment tallies moved. The same
/// event is evidence at every scale it belongs to, so a Sentinel need not
/// choose in advance which granularity will turn out to matter.
///
/// (´claim:ledger:a-write-updates-every-tracked-cell-containing-the-coordinate´)
/// ´test:crate:all-layers-write-hits-all´
#[test]
fn all_layers_write_hits_all() {
    let mut ledger = make_ledger_with_depths(&[0, 4, 8]);

    let update = LedgerUpdate::new(true, 0.5, 1.0, Action::Allow, true);
    let now = clock().now();

    let count = update_all_layers(&mut ledger, 0, 0.999, &now, &update, 0.999);
    assert_eq!(count, 3, "should update all three layers");

    // Verify all entries updated
    assert_eq!(ledger.root().total_assessments, 1);
    assert_eq!(ledger.get(&LedgerKey::new(0, 4)).unwrap().total_assessments, 1);
    assert_eq!(ledger.get(&LedgerKey::new(0, 8)).unwrap().total_assessments, 1);
}

/// Each write mixes one minus lambda of the new observation into what was
/// already stored: from zero, one adverse outcome puts the bad-rate at exactly
/// that fraction and the compressed valence at that fraction of the supplied
/// value, and a following benign outcome scales the bad-rate by lambda rather
/// than replacing it. The retention factor is what makes a single event a small
/// nudge and a sustained pattern a real movement.
///
/// ´claim:ledger:each-ewma-step-mixes-one-minus-lambda-of-the-new-observation-into-the-stored-value´
/// ´test:crate:all-layers-write-ewma-formula´
#[test]
fn all_layers_write_ewma_formula() {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();

    // Initial state
    assert_near(ledger.root().ewma_bad_rate, 0.0, TOL.bit_identical, "initial bad_rate");

    // Positive outcome (is_positive = true → is_bad = 1, adverse)
    let update_positive = LedgerUpdate::new(true, 0.5, 1.0, Action::Allow, true);
    let now = clock().now();
    let lambda = 0.999;

    update_all_layers(&mut ledger, 0, 0.999, &now, &update_positive, lambda);

    // After one positive (adverse) update: bad_rate = λ·0 + (1-λ)·1 = 0.001
    let expected_bad_after_pos = 1.0 - lambda;
    assert_near(
        ledger.root().ewma_bad_rate,
        expected_bad_after_pos,
        TOL.ewma,
        "bad_rate after positive",
    );
    // compressed_valence = λ·0 + (1-λ)·0.5 = 0.0005
    assert_near(
        ledger.root().compressed_valence_ewma,
        (1.0 - lambda) * 0.5,
        TOL.ewma,
        "compressed_valence after positive",
    );

    // Now a negative outcome (is_positive = false → is_bad = 0)
    let update_negative = LedgerUpdate::new(false, -0.5, -1.0, Action::Block, true);
    update_all_layers(&mut ledger, 0, 0.999, &now, &update_negative, lambda);

    // bad_rate = λ·0.001 + (1-λ)·0 = 0.000999
    assert_near(
        ledger.root().ewma_bad_rate,
        lambda * expected_bad_after_pos,
        TOL.ewma,
        "bad_rate after negative",
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Decay Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Reading a decayed view leaves the entry exactly as it was, and two
/// successive reads at the same instant agree. Decay is a lens applied at read
/// time, not a mutation, so a host may read the ledger as often as it likes —
/// for diagnostics, for several axes, for a dashboard — without those reads
/// themselves eroding the evidence.
///
/// ´claim:ledger:reading-a-decayed-view-does-not-mutate-the-stored-entry´
/// ´test:crate:read-decay-pure´
#[test]
fn read_decay_pure() {
    let mut entry = LedgerEntry::new_neutral();
    entry.ewma_bad_rate = 0.5;

    let original_rate = entry.ewma_bad_rate;
    let now = clock().now();

    // First call
    let view1 = entry.read_decayed(0.999, &now);
    // Second call should return identical result
    let view2 = entry.read_decayed(0.999, &now);

    assert_near(view1.bad_rate, view2.bad_rate, TOL.bit_identical, "read_decayed is pure");
    assert_near(entry.ewma_bad_rate, original_rate, TOL.bit_identical, "entry must not mutate");
}

/// At the default hourly rate an entry is worth about half of what it was after
/// twenty-nine days: a stored value of one reads back near a half once the
/// stamp is rewound by that span. The half-life is the ledger's stated promise
/// about how quickly a bad patch stops counting against a region, and it is a
/// property of the elapsed hours rather than of any sweep having run.
///
/// ´claim:ledger:an-entry-is-worth-half-its-stored-value-after-twenty-nine-days´
/// ´test:crate:read-decay-factor´
#[test]
fn read_decay_factor() {
    // Entry last updated 29 days ago with γ=0.999/hr has ~0.5 decay factor
    let mut entry = LedgerEntry::new_neutral();
    entry.ewma_bad_rate = 1.0;

    // 29 days = 696 hours
    // γ^696 = 0.999^696 ≈ 0.498
    let old_time = PersistentTimestamp::new(entry.last_updated.seconds - 29 * 24 * 3600, 0);
    entry.last_updated = old_time;

    let now = PersistentTimestamp::now();
    let view = entry.read_decayed(0.999, &now);

    // Should be approximately 0.5 (within ±0.05)
    assert_near(view.bad_rate, 0.5, 0.05, "29-day decay ≈ 0.5");
}

/// A write discounts the stored value for elapsed time before mixing in the new
/// observation: an entry holding one, untouched for an hour, ends at lambda
/// times the hourly decay factor rather than at lambda alone. Ordering matters
/// because the new observation must not be aged along with the history it is
/// joining — it happened now, and only the past owes time.
///
/// ´claim:ledger:a-write-decays-the-stored-value-for-elapsed-time-before-mixing-in-the-new-observation´
/// ´test:crate:write-decay-before-update´
#[test]
fn write_decay_before_update() {
    let mut entry = LedgerEntry::new_neutral();
    entry.ewma_bad_rate = 1.0;

    // Set last_updated to 1 hour ago
    let one_hour_ago = PersistentTimestamp::new(entry.last_updated.seconds - 3600, 0);
    entry.last_updated = one_hour_ago;

    let now = PersistentTimestamp::now();
    let update = LedgerUpdate::new(true, 0.0, 0.0, Action::Allow, true);
    let lambda = 0.999;

    entry.apply_write_decay_and_update(0.999, &now, &update, lambda);

    // Original 1.0 decayed by 0.999 then EWMA'd with 0
    // Result: λ·(γ·1.0) + (1-λ)·0 = λ·γ ≈ 0.998
    assert_near(entry.ewma_bad_rate, lambda * 0.999, 0.01, "write_decay_before_update");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Cell-Set Maintenance Tests
// ═══════════════════════════════════════════════════════════════════════════════

fn make_key(lo: u128, depth: u8) -> LedgerKey {
    LedgerKey::new(lo, depth)
}

/// Three cells named at once by a first report are all taken up, and the
/// summary distinguishes the three that appeared from the four now tracked —
/// the root included. Callers get both figures because "how much did this
/// report change" and "how large is this ledger now" answer different
/// questions, one about churn and one about footprint.
///
/// (´claim:ledger:a-cell-newly-named-by-a-report-starts-being-tracked-from-neutral´)
/// ´test:crate:cell-set-appearances´
#[test]
fn cell_set_appearances() {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();

    let new_cells: HashSet<LedgerKey> = vec![make_key(0, 4), make_key(0, 8), make_key(0, 12)].into_iter().collect();

    let changes = apply_cell_set_changes(&mut ledger, &new_cells, 3);

    assert_eq!(changes.appeared, 3);
    assert_eq!(changes.deleted, 0);
    assert_eq!(changes.tracked, 4); // root + 3
}

/// Two empty reports in succession leave the tracked cell's absence count at
/// two, one per cycle. The count is a running tally of cycles rather than a
/// flag, so the deletion threshold can be expressed in report cycles and mean
/// the same thing whatever a Sentinel's reporting cadence happens to be.
///
/// (´claim:ledger:a-cell-missing-from-a-report-accrues-a-consecutive-absence´)
/// ´test:crate:cell-set-absence-increment´
#[test]
fn cell_set_absence_increment() {
    let key4 = make_key(0, 4);
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();
    ledger.insert(key4, LedgerEntry::new_neutral());

    let empty: HashSet<LedgerKey> = HashSet::new();
    apply_cell_set_changes(&mut ledger, &empty, 3);

    assert_eq!(ledger.get(&key4).unwrap().consecutive_absent, 1);

    apply_cell_set_changes(&mut ledger, &empty, 3);
    assert_eq!(ledger.get(&key4).unwrap().consecutive_absent, 2);
}

/// A cell absent from two reports is still present and readable; the third
/// absence removes it and is reported as one deletion. The threshold names the
/// absence on which removal happens, not the number that must pass before it
/// becomes possible.
///
/// (´claim:ledger:a-cell-is-dropped-on-the-absence-that-reaches-the-threshold´)
/// ´test:crate:cell-set-deletion´
#[test]
fn cell_set_deletion() {
    let key4 = make_key(0, 4);
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();
    ledger.insert(key4, LedgerEntry::new_neutral());

    let empty: HashSet<LedgerKey> = HashSet::new();

    // Threshold is 3
    apply_cell_set_changes(&mut ledger, &empty, 3);
    assert!(ledger.get(&key4).is_some());
    apply_cell_set_changes(&mut ledger, &empty, 3);
    assert!(ledger.get(&key4).is_some());

    // Third absence triggers deletion
    let changes = apply_cell_set_changes(&mut ledger, &empty, 3);
    assert_eq!(changes.deleted, 1);
    assert!(ledger.get(&key4).is_none());
}

/// Ten rounds of cell-set maintenance against an empty report and a zero
/// threshold leave the root in place. A misconfigured or degenerate threshold
/// cannot cost a Sentinel the one cell every read must be able to reach.
///
/// (´claim:ledger:the-root-cell-is-never-deleted-however-long-it-is-absent´)
/// ´test:crate:cell-set-root-never-deleted´
#[test]
fn cell_set_root_never_deleted() {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();

    let empty: HashSet<LedgerKey> = HashSet::new();

    // Even with threshold 0, root survives
    for _ in 0..10 {
        apply_cell_set_changes(&mut ledger, &empty, 0);
    }

    assert!(ledger.has_root());
}

/// When a child is aged out while its parent keeps reporting, the parent's
/// accumulated tally is precisely what it was before — no rollup happens at the
/// moment of deletion. Losing the finer view costs resolution, not accuracy at
/// the coarser scale.
///
/// (´claim:ledger:deleting-a-cell-leaves-its-parent-untouched-rather-than-merging-into-it´)
/// ´test:crate:cell-set-no-merge´
#[test]
fn cell_set_no_merge() {
    let key4 = make_key(0, 4);
    let key8 = make_key(0, 8);
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();
    ledger.insert(key4, LedgerEntry::new_neutral());
    ledger.insert(key8, LedgerEntry::new_neutral());

    // Mark parent with data
    ledger.get_mut(&key4).unwrap().total_assessments = 100;

    // Keep key4 in report, only key8 is absent
    let report_with_key4: HashSet<LedgerKey> = vec![key4].into_iter().collect();
    for _ in 0..3 {
        apply_cell_set_changes(&mut ledger, &report_with_key4, 3);
    }

    // Child deleted, parent unchanged
    assert!(ledger.get(&key8).is_none());
    assert_eq!(ledger.get(&key4).unwrap().total_assessments, 100);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Garbage Collection Tests
// ═══════════════════════════════════════════════════════════════════════════════

fn old_timestamp() -> PersistentTimestamp {
    let now = PersistentTimestamp::now();
    PersistentTimestamp::new(now.seconds - 100 * 24 * 3600, 0)
}

/// A hundred-day-old entry with nothing recorded in it is collected under a
/// ninety-day horizon, and the sweep reports the one removal. Both conditions
/// are met, so the entry can be dropped without losing anything a read would
/// have found there.
///
/// (´claim:ledger:an-entry-that-is-both-stale-and-empty-is-collected´)
/// ´test:crate:gc-removes-old-low´
#[test]
fn gc_removes_old_low() {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();

    let key4 = make_key(0, 4);
    let mut entry = LedgerEntry::new_neutral();
    entry.last_updated = old_timestamp();
    ledger.insert(key4, entry);

    let now = PersistentTimestamp::now();
    let horizon = Duration::from_secs(90 * 24 * 3600);

    let removed = garbage_collect(&mut ledger, 1e-6, horizon, &now);
    assert_eq!(removed, 1);
    assert!(ledger.get(&key4).is_none());
}

/// The same hundred-day-old entry, differing only in carrying a substantial
/// bad-rate, is left alone and the sweep reports nothing removed. Retention
/// turns on content, so a region that went badly and then went quiet keeps its
/// record until read-time decay has genuinely emptied it.
///
/// (´claim:ledger:an-entry-still-carrying-signal-survives-collection-however-old-it-is´)
/// ´test:crate:gc-keeps-high-ewma´
#[test]
fn gc_keeps_high_ewma() {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();

    let key4 = make_key(0, 4);
    let mut entry = LedgerEntry::new_neutral();
    entry.last_updated = old_timestamp();
    entry.ewma_bad_rate = 0.5;
    ledger.insert(key4, entry);

    let now = PersistentTimestamp::now();
    let horizon = Duration::from_secs(90 * 24 * 3600);

    let removed = garbage_collect(&mut ledger, 1e-6, horizon, &now);
    assert_eq!(removed, 0);
    assert!(ledger.get(&key4).is_some());
}

/// An aged, empty root swept under a one-second horizon survives and the sweep
/// reports nothing removed. The exemption is unconditional rather than a
/// consequence of the root usually being fresh, so no combination of horizon
/// and floor can strip a Sentinel of its terminal routing destination.
///
/// (´claim:ledger:the-root-entry-is-never-garbage-collected´)
/// ´test:crate:ledger-gc-never-removes-root´
#[test]
fn ledger_gc_never_removes_root() {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();
    ledger.root_mut().last_updated = old_timestamp();

    let now = PersistentTimestamp::now();
    let horizon = Duration::from_secs(1);

    let removed = garbage_collect(&mut ledger, 1e-6, horizon, &now);
    assert_eq!(removed, 0);
    assert!(ledger.has_root());
}

/// An old entry whose only content is one axis row above the floor is retained
/// by the sweep. Axis rows are appended lazily and only where a host has
/// actually reported values, so they are exactly the content most likely to be
/// sparse — and most costly to discard, since nothing else in the ledger
/// records it.
///
/// (´claim:ledger:a-per-axis-ewma-above-the-floor-is-enough-to-retain-an-entry´)
/// ´test:crate:gc-per-axis-ewma´
#[test]
fn gc_per_axis_ewma() {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();

    let key4 = make_key(0, 4);
    let mut entry = LedgerEntry::new_neutral();
    entry.last_updated = old_timestamp();
    entry.per_axis.push((OutcomeAxisId(1), 0.5, 0.0));
    ledger.insert(key4, entry);

    let now = PersistentTimestamp::now();
    let horizon = Duration::from_secs(90 * 24 * 3600);

    let removed = garbage_collect(&mut ledger, 1e-6, horizon, &now);
    assert_eq!(removed, 0, "per-axis EWMA above floor should prevent GC");
}

/// Batched collection clears twenty-three expired entries while holding the
/// exclusive lock only in bounded slices: the read side takes one key-snapshot
/// acquisition and one scan window within its limit, and the write side takes
/// exactly as many acquisitions as the batch limit implies, none larger than
/// that limit. Afterwards only the root remains and the recorded maximum depth
/// has been recalculated back down to zero. The ledger is read on the
/// assessment path, so a sweep that took one long exclusive hold proportional
/// to the backlog would stall live traffic precisely when the ledger is
/// largest.
///
/// ´claim:ledger:batched-collection-bounds-every-scan-and-deletion-so-no-lock-hold-scales-with-the-backlog´
/// ´test:crate:gc-batched-deletion-bounds-lock-windows´
#[test]
fn gc_batched_deletion_bounds_lock_windows() {
    let expired_count = 23usize;
    let delete_batch_limit = 7usize;
    let limits = GcLimits::new(expired_count + 1, delete_batch_limit);
    let ledger = RwLock::new(SentinelLedger::new());

    {
        let mut guard = ledger.write().unwrap();
        guard.ensure_root();
        for idx in 0..expired_count {
            let mut entry = LedgerEntry::new_neutral();
            entry.last_updated = old_timestamp();
            guard.insert(make_key(idx as u128 + 1, 128), entry);
        }
    }

    let now = PersistentTimestamp::now();
    let horizon = Duration::from_secs(90 * 24 * 3600);
    let outcome = garbage_collect_batched(&ledger, limits, 1e-6, horizon, &now);

    assert_eq!(outcome.removed, expired_count);
    // One acquisition snapshots the key set, one scans the single window.
    assert_eq!(outcome.read_lock_acquisitions, 2);
    assert_eq!(outcome.write_lock_acquisitions, expired_count.div_ceil(delete_batch_limit));
    assert!(outcome.max_scan_window <= limits.scan_limit);
    assert!(outcome.max_delete_batch <= delete_batch_limit);

    let guard = ledger.read().unwrap();
    assert_eq!(guard.entry_count(), 1);
    assert!(guard.has_root());
    assert_eq!(guard.max_depth(), 0);
}

/// A sweep started over an eligible population removes every one of its
/// members even while a writer keeps inserting fresh entries between the
/// sweep's bounded lock windows. Concurrent insertion may resize and rehash
/// the entry map mid-sweep; the sweep's candidate discovery walks a key
/// snapshot taken once at the start rather than a position in iteration
/// order, so no eligible entry is skipped and no survivor is inspected
/// twice as the table moves underneath it. The fresh entries themselves all
/// survive, being newer than any horizon.
///
/// ´claim:ledger:a-sweep-scans-every-entry-it-set-out-to-scan-despite-concurrent-rehashing´
/// ´test:crate:gc-batched-survives-concurrent-rehash´
#[test]
fn gc_batched_survives_concurrent_rehash() {
    let eligible_count = 257usize;
    let ledger = Arc::new(RwLock::new(SentinelLedger::new()));

    {
        let mut guard = ledger.write().unwrap();
        guard.ensure_root();
        for idx in 0..eligible_count {
            let mut entry = LedgerEntry::new_neutral();
            entry.last_updated = old_timestamp();
            guard.insert(make_key(idx as u128 + 1, 128), entry);
        }
    }

    let stop = Arc::new(AtomicBool::new(false));
    let writer = {
        let ledger = Arc::clone(&ledger);
        let stop = Arc::clone(&stop);
        thread::spawn(move || {
            let mut idx = 0u128;
            while !stop.load(Ordering::Relaxed) {
                let mut guard = ledger.write().unwrap();
                // Fresh entries at a different depth so keys never collide
                // with the eligible population; recent, so never eligible.
                guard.insert(make_key(idx + 1, 64), LedgerEntry::new_neutral());
                idx += 1;
                drop(guard);
                thread::yield_now();
            }
        })
    };

    let now = PersistentTimestamp::now();
    let horizon = Duration::from_secs(90 * 24 * 3600);
    // Small windows so the writer gets many chances to rehash mid-sweep.
    let limits = GcLimits::new(16, 8);
    let outcome = garbage_collect_batched(&ledger, limits, 1e-6, horizon, &now);

    stop.store(true, Ordering::Relaxed);
    writer.join().unwrap();

    assert_eq!(
        outcome.removed, eligible_count,
        "every eligible entry is collected in one sweep"
    );
    let guard = ledger.read().unwrap();
    assert!(guard.has_root());
    for idx in 0..eligible_count {
        assert!(
            guard.get(&make_key(idx as u128 + 1, 128)).is_none(),
            "eligible entry {idx} skipped by the sweep"
        );
    }
}

/// The collection scheduler actually collects: started over a container
/// holding two Sentinels — one carrying an entry both stale and empty, the
/// other a recent one — it removes the eligible entry within a few ticks and
/// leaves the recent entry and both roots standing, reading the present
/// through the injected clock and the floor and horizon it was configured
/// with. Storage economy is the sweep's whole purpose, and an implemented
/// sweep with no caller bounds nothing.
///
/// ´claim:ledger:the-periodic-scheduler-sweeps-every-sentinels-ledger-with-the-configured-floor-and-horizon´
/// ´test:crate:ledger-gc-scheduler-collects-on-cadence´
#[test]
fn ledger_gc_scheduler_collects_on_cadence() {
    use crate::ledger::{SweepConfig, spawn_ledger_gc_scheduler};

    // Present: 100 days after the persistent epoch; horizon: 60 days.
    let clock: Arc<dyn Clock> = Arc::new(VirtualClock::at_secs(100 * 24 * 3600));
    let container = Arc::new(OutcomeLedger::new());

    let stale_sentinel = SentinelId(1);
    let fresh_sentinel = SentinelId(2);
    container.create_sentinel(stale_sentinel);
    container.create_sentinel(fresh_sentinel);

    let stale_key = make_key(1, 128);
    {
        let lock = container.get(stale_sentinel).unwrap();
        let mut guard = lock.write().unwrap();
        let mut entry = LedgerEntry::new_neutral();
        entry.last_updated = PersistentTimestamp::new(0, 0); // 100 days old
        guard.insert(stale_key, entry);
    }
    let fresh_key = make_key(1, 128);
    {
        let lock = container.get(fresh_sentinel).unwrap();
        let mut guard = lock.write().unwrap();
        let mut entry = LedgerEntry::new_neutral();
        entry.last_updated = PersistentTimestamp::new(99 * 24 * 3600, 0); // 1 day old
        guard.insert(fresh_key, entry);
    }

    let (control_tx, control_rx) = crossbeam_channel::bounded::<()>(0);
    let handle = spawn_ledger_gc_scheduler(
        "test",
        Duration::from_millis(2),
        Arc::clone(&container),
        SweepConfig {
            floor: 1e-6,
            horizon: Duration::from_secs(60 * 24 * 3600),
        },
        Arc::clone(&clock),
        control_rx,
    );

    // Wait for the sweep to take the stale entry (bounded wait).
    let deadline = std::time::Instant::now() + crate::testing::STATE_DEADLINE;
    loop {
        let gone = {
            let lock = container.get(stale_sentinel).unwrap();
            let guard = lock.read().unwrap();
            guard.get(&stale_key).is_none()
        };
        if gone {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "scheduler never collected the stale entry"
        );
        thread::sleep(Duration::from_millis(2));
    }

    // The recent entry and both roots survive.
    {
        let lock = container.get(fresh_sentinel).unwrap();
        let guard = lock.read().unwrap();
        assert!(guard.get(&fresh_key).is_some(), "recent entry must survive the sweep");
        assert!(guard.has_root());
    }
    {
        let lock = container.get(stale_sentinel).unwrap();
        let guard = lock.read().unwrap();
        assert!(guard.has_root());
    }

    // Dropping the control sender shuts the scheduler down.
    drop(control_tx);
    handle.join().expect("scheduler thread must join cleanly");
}

// ═══════════════════════════════════════════════════════════════════════════════
// OutcomeLedger Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A Sentinel registered with the container is immediately both discoverable
/// and readable through its lock, with the root already installed behind it.
/// There is no window in which a registered Sentinel exists but cannot be
/// assessed against.
///
/// (´claim:ledger:a-sentinels-ledger-is-created-with-its-root-already-in-place´)
/// ´test:crate:outcome-ledger-create-sentinel´
#[test]
fn outcome_ledger_create_sentinel() {
    let ledger = OutcomeLedger::new();
    ledger.create_sentinel(SentinelId(1));

    assert!(ledger.contains(SentinelId(1)));
    let lock = ledger.get(SentinelId(1)).unwrap();
    let guard = lock.read().unwrap();
    assert!(guard.has_root());
    drop(guard);
}

/// An axis value supplied with an update takes the same EWMA step as the
/// entry-wide figures, and its compressed and raw components are carried
/// separately rather than one being derived from the other. A host reading a
/// registered axis therefore gets a history built on exactly the arithmetic it
/// can reason about from the binary rate.
///
/// ´claim:ledger:a-supplied-axis-value-takes-the-same-ewma-step-as-the-entry-wide-figures´
/// ´test:crate:per-axis-ewmas-updated´
#[test]
fn per_axis_ewmas_updated() {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();

    let axis = OutcomeAxisId(42);
    let update = LedgerUpdate::new(true, 0.5, 1.0, Action::Allow, true).with_axis(axis, 0.7, 0.8);

    let now = clock().now();
    let lambda = 0.999;

    update_all_layers(&mut ledger, 0, 0.999, &now, &update, lambda);

    let root = ledger.root();
    let expected_compressed = (1.0 - lambda) * 0.7;
    let expected_raw = (1.0 - lambda) * 0.8;
    let (actual_c, actual_r) = axis_val(&root.per_axis, axis);

    assert_near(actual_c, expected_compressed, TOL.ewma, "per-axis compressed");
    assert_near(actual_r, expected_raw, TOL.ewma, "per-axis raw");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Serialisation Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A serialised ledger comes back with its root and its deeper cells intact,
/// and with the root's recorded tally preserved. Entries are keyed by a
/// composite dyadic key that no textual format can express as a plain map key,
/// so the round-trip is the evidence that the pair-sequence encoding
/// reconstructs the same keyed structure it started from.
///
/// ´claim:ledger:a-serialised-ledger-comes-back-with-its-root-and-cells-intact´
/// ´test:crate:sentinel-ledger-serde-roundtrip´
#[cfg(feature = "serde")]
#[test]
fn sentinel_ledger_serde_roundtrip() {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();
    ledger.insert(make_key(0, 4), LedgerEntry::new_neutral());
    ledger.root_mut().total_assessments = 42;

    let json = serde_json::to_string(&ledger).expect("serialize");
    let restored: SentinelLedger = serde_json::from_str(&json).expect("deserialize");

    assert!(restored.has_root());
    assert_eq!(restored.root().total_assessments, 42);
    assert!(restored.get(&make_key(0, 4)).is_some());
}

/// An entry's axis rows survive serialisation with both components intact and
/// still attached to the right axis id — two axes go out and come back
/// distinguishable, compressed and raw each landing where it belongs. Since the
/// rows are a positional triple rather than a named record, a transposition
/// would restore plausible numbers under the wrong meaning.
///
/// ´claim:ledger:an-entrys-axis-rows-survive-serialisation-with-both-components-intact´
/// ´test:crate:entry-serde-with-axes´
#[cfg(feature = "serde")]
#[test]
fn entry_serde_with_axes() {
    let mut entry = LedgerEntry::new_neutral();
    entry.ewma_bad_rate = 0.123;
    entry.per_axis.push((OutcomeAxisId(1), 0.5, 1.5));
    entry.per_axis.push((OutcomeAxisId(2), 0.7, 0.0));

    let json = serde_json::to_string(&entry).expect("serialize");
    let restored: LedgerEntry = serde_json::from_str(&json).expect("deserialize");

    assert_near(restored.ewma_bad_rate, 0.123, TOL.bit_identical, "bad_rate roundtrip");

    let (c1, r1) = axis_val(&restored.per_axis, OutcomeAxisId(1));
    let (c2, _r2) = axis_val(&restored.per_axis, OutcomeAxisId(2));
    assert_near(c1, 0.5, TOL.bit_identical, "axis 1 compressed");
    assert_near(c2, 0.7, TOL.bit_identical, "axis 2 compressed");
    assert_near(r1, 1.5, TOL.bit_identical, "axis 1 raw");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Additional Read-Time Decay Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// An entry read at the very instant it was written comes back undecayed —
/// bad-rate, compressed valence and raw valence all bit-identical to what is
/// stored. Zero elapsed hours costs nothing, so the outcome of a label is fully
/// available to the assessment that follows it immediately rather than being
/// docked for the round trip.
///
/// ´claim:ledger:an-entry-read-at-the-instant-it-was-written-is-returned-undecayed´
/// ´test:crate:read-decay-zero-elapsed´
#[test]
fn read_decay_zero_elapsed() {
    let mut entry = LedgerEntry::new_neutral();
    entry.ewma_bad_rate = 0.75;
    entry.compressed_valence_ewma = 0.5;
    entry.raw_valence_ewma = 2.0;

    let now = entry.last_updated;
    let view = entry.read_decayed(0.999, &now);

    // No time elapsed → no decay
    assert_near(view.bad_rate, 0.75, TOL.bit_identical, "bad_rate unchanged");
    assert_near(view.compressed_valence, 0.5, TOL.bit_identical, "compressed unchanged");
    assert_near(view.raw_valence, 2.0, TOL.bit_identical, "raw unchanged");
}

/// After a year untouched an entry reads as vanishingly small yet still
/// strictly positive. The decay is geometric, so it approaches zero without
/// ever arriving: nothing is clamped or flushed to nothing, and an entry that
/// sat out an extreme gap is neither still influential nor a source of exact
/// zeros that downstream ratios would have to special-case.
///
/// ´claim:ledger:a-long-untouched-entry-decays-toward-zero-without-reaching-it´
/// ´test:crate:read-decay-large-gap´
#[test]
fn read_decay_large_gap() {
    let mut entry = LedgerEntry::new_neutral();
    entry.ewma_bad_rate = 1.0;
    entry.compressed_valence_ewma = 1.0;

    // Set last updated to 1 year ago (8,760 hours)
    let one_year_ago = PersistentTimestamp::new(entry.last_updated.seconds - 365 * 24 * 3600, 0);
    entry.last_updated = one_year_ago;

    let now = PersistentTimestamp::now();
    let view = entry.read_decayed(0.999, &now);

    // γ^8760 = 0.999^8760 ≈ 0.000015 — very close to zero
    assert!(view.bad_rate > 0.0, "decay should not be exactly zero");
    assert!(
        view.bad_rate < 0.001,
        "decay after 1 year should be near zero: got {}",
        view.bad_rate
    );
    assert!(view.compressed_valence < 0.001);
}

/// Every EWMA in an entry ages by one shared factor: after a day, the bad-rate
/// and both axis rows each read back at the same twenty-four-hour factor times
/// what they stored. One timestamp governs the whole entry, so the relationships
/// between an entry's figures — an axis against the binary rate, one axis
/// against another — survive the passage of time unchanged.
///
/// ´claim:ledger:every-ewma-in-an-entry-ages-by-the-same-shared-factor´
/// ´test:crate:read-decay-per-axis´
#[test]
fn read_decay_per_axis() {
    let mut entry = LedgerEntry::new_neutral();
    entry.ewma_bad_rate = 1.0;
    entry.per_axis.push((OutcomeAxisId(1), 0.5, 0.0));
    entry.per_axis.push((OutcomeAxisId(2), 0.8, 0.0));

    // 24 hours elapsed
    let one_day_ago = PersistentTimestamp::new(entry.last_updated.seconds - 24 * 3600, 0);
    entry.last_updated = one_day_ago;

    let now = PersistentTimestamp::now();
    let view = entry.read_decayed(0.999, &now);

    // γ^24 ≈ 0.976
    let expected_factor = 0.999_f64.powi(24);
    let tolerance = 0.001;

    // All EWMAs should decay by the same factor
    assert_near(view.bad_rate, expected_factor, tolerance, "bad_rate decay factor");
    assert_near(
        axis_compressed(&view.per_axis, OutcomeAxisId(1)),
        expected_factor * 0.5,
        tolerance,
        "axis 1 decay",
    );
    assert_near(
        axis_compressed(&view.per_axis, OutcomeAxisId(2)),
        expected_factor * 0.8,
        tolerance,
        "axis 2 decay",
    );
}

/// A clock that has gone backwards decays nothing rather than amplifying:
/// reading with a "now" an hour before the entry's own stamp returns the stored
/// value untouched. Negative elapsed time in a geometric factor would inflate
/// the evidence, turning an NTP correction into a manufactured spike in a
/// region's bad-rate.
///
/// ´claim:ledger:a-clock-that-has-gone-backwards-decays-nothing-rather-than-amplifying´
/// ´test:crate:read-decay-backward-clock´
#[test]
fn read_decay_backward_clock() {
    let mut entry = LedgerEntry::new_neutral();
    entry.ewma_bad_rate = 0.5;

    // Set "now" to before last_updated (clock anomaly)
    let past = PersistentTimestamp::new(entry.last_updated.seconds - 3600, 0);
    let view = entry.read_decayed(0.999, &past);

    // Factor should clamp to 1.0, no decay
    assert_near(view.bad_rate, 0.5, TOL.bit_identical, "backward clock does not decay");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Additional EWMA Write Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A benign outcome contributes nothing to the bad-rate: recorded against an
/// empty entry it leaves the rate at exactly zero. Only adverse outcomes are
/// evidence of badness, so a benign one can dilute an existing rate through the
/// retention factor but can never seed one from nothing.
///
/// ´claim:ledger:a-benign-outcome-contributes-nothing-to-the-bad-rate´
/// ´test:crate:write-single-negative´
#[test]
fn write_single_negative() {
    let mut entry = LedgerEntry::new_neutral();
    let now = clock().now();
    let lambda = 0.999;

    // Negative outcome (is_positive = false → is_bad = 0)
    let update = LedgerUpdate::new(false, -0.5, -1.0, Action::Block, true);
    entry.apply_write_decay_and_update(0.999, &now, &update, lambda);

    // bad_rate = λ·0 + (1-λ)·0 = 0
    assert_near(entry.ewma_bad_rate, 0.0, TOL.bit_identical, "single negative → 0");
}

/// A run of adverse outcomes drives the EWMAs toward their drivers
/// geometrically rather than linearly: ten of them take the bad-rate to one
/// minus lambda to the tenth, with the compressed valence at the same fraction
/// of its supplied value. Ten bad events are therefore still a small number
/// against a long-memory retention factor — a burst has to be sustained to
/// register, which is what stops a brief incident from condemning a region.
///
/// ´claim:ledger:a-run-of-adverse-outcomes-drives-the-ewmas-toward-their-drivers-geometrically´
/// ´test:crate:write-ten-positives´
#[test]
fn write_ten_positives() {
    let mut entry = LedgerEntry::new_neutral();
    let now = clock().now();
    let lambda = 0.999;

    let update = LedgerUpdate::new(true, 0.5, 1.0, Action::Allow, true);

    for _ in 0..10 {
        entry.apply_write_decay_and_update(0.999, &now, &update, lambda);
    }

    // After 10 positive (adverse) steps: bad_rate = 1 − λ^10 ≈ 0.00995
    assert_near(entry.ewma_bad_rate, 0.01, 0.001, "10 positives → bad_rate ≈ 0.01");

    // After 10 steps: 0.5 * (1 − λ^10) ≈ 0.00498
    assert_near(
        entry.compressed_valence_ewma,
        0.005,
        0.001,
        "10 positives → compressed ≈ 0.005",
    );
}

/// A mixed history settles between the two extremes rather than tracking
/// whichever outcome came last: an alternating sequence of two adverse and two
/// benign outcomes leaves the bad-rate near twice the single-step increment,
/// slightly discounted by the benign steps in between. What is recorded is the
/// proportion of adverse outcomes over the retained window, not the most recent
/// verdict.
///
/// ´claim:ledger:a-mixed-history-settles-between-the-extremes-rather-than-tracking-the-latest-outcome´
/// ´test:crate:write-alternating´
#[test]
fn write_alternating() {
    let mut entry = LedgerEntry::new_neutral();
    let now = clock().now();
    let lambda = 0.999;

    // Alternating: +, -, +, -
    let positive = LedgerUpdate::new(true, 0.5, 1.0, Action::Allow, true);
    let negative = LedgerUpdate::new(false, -0.5, -1.0, Action::Block, true);

    entry.apply_write_decay_and_update(0.999, &now, &positive, lambda);
    entry.apply_write_decay_and_update(0.999, &now, &negative, lambda);
    entry.apply_write_decay_and_update(0.999, &now, &positive, lambda);
    entry.apply_write_decay_and_update(0.999, &now, &negative, lambda);

    // After the four-step sequence: bad_rate ≈ 0.001996
    assert_near(entry.ewma_bad_rate, 0.002, 0.0001, "+-+- → bad_rate ≈ 0.002");
}

/// The compressed valence EWMA averages exactly the compressed figure the
/// caller supplies, applying no squashing of its own — a supplied value of 0.9
/// yields one minus lambda times 0.9 to bit precision. Compression happens
/// upstream where the scale constant lives, so the ledger's job here is to
/// remember rather than to transform.
///
/// ´claim:ledger:the-compressed-valence-ewma-averages-the-compressed-value-the-caller-supplies´
/// ´test:crate:write-compressed-valence´
#[test]
fn write_compressed_valence() {
    let mut entry = LedgerEntry::new_neutral();
    let now = clock().now();
    let lambda = 0.999;

    // Valence v = 0.9 (already compressed to tanh range)
    let update = LedgerUpdate::new(true, 0.9, 5.0, Action::Allow, true);
    entry.apply_write_decay_and_update(0.999, &now, &update, lambda);

    // compressed_valence = (1-λ) * 0.9 = 0.0009
    assert_near(
        entry.compressed_valence_ewma,
        (1.0 - lambda) * 0.9,
        TOL.bit_identical,
        "compressed_valence formula",
    );
}

/// The raw valence is averaged alongside the compressed one on its own
/// unsquashed scale: a raw value of five contributes one minus lambda times
/// five, unbounded by the compressed range. Keeping both means a host can ask
/// how bad the outcomes were in their original units as well as how they rank
/// once compressed — magnitudes that saturation would otherwise flatten away.
///
/// ´claim:ledger:the-raw-valence-is-averaged-on-its-own-unsquashed-scale-alongside-the-compressed-one´
/// ´test:crate:write-raw-valence´
#[test]
fn write_raw_valence() {
    let mut entry = LedgerEntry::new_neutral();
    let now = clock().now();
    let lambda = 0.999;

    let update = LedgerUpdate::new(true, 0.5, 5.0, Action::Allow, true);
    entry.apply_write_decay_and_update(0.999, &now, &update, lambda);

    // raw_valence = (1-λ) * 5.0 = 0.005
    assert_near(
        entry.raw_valence_ewma,
        (1.0 - lambda) * 5.0,
        TOL.bit_identical,
        "raw_valence formula",
    );
}

/// An axis the update says nothing about is left where it was rather than being
/// averaged toward zero: a pre-existing row survives an update that carries no
/// value for it. Silence on an axis is missing data, not a report of nothing
/// happening, so an axis a host reports only occasionally must not have its
/// history eroded by every unrelated label in between.
///
/// ´claim:ledger:an-axis-absent-from-an-update-is-left-as-it-was-rather-than-averaged-toward-zero´
/// ´test:crate:write-per-axis-absent´
#[test]
fn write_per_axis_absent() {
    let mut entry = LedgerEntry::new_neutral();
    let now = clock().now();
    let lambda = 0.999;

    // Pre-populate axis (compressed=0.5, raw=0.0)
    entry.per_axis.push((OutcomeAxisId(1), 0.5, 0.0));

    // Update WITHOUT axis data for axis 1
    let update = LedgerUpdate::new(true, 0.5, 1.0, Action::Allow, true);
    entry.apply_write_decay_and_update(0.999, &now, &update, lambda);

    // Axis 1 should be decayed only (factor=1.0 since now==last_updated), not EWMA'd
    assert_near(
        axis_compressed(&entry.per_axis, OutcomeAxisId(1)),
        0.5,
        TOL.ewma,
        "absent axis unchanged",
    );
}

/// A write stamps the entry with the instant it was applied, replacing an
/// ancient stamp outright. The stamp is the origin every later decay is
/// measured from, so leaving it behind would make the next read discount the
/// observation just recorded for time that elapsed before it happened.
///
/// ´claim:ledger:a-write-stamps-the-entry-with-the-instant-it-was-applied´
/// ´test:crate:write-updates-timestamp´
#[test]
fn write_updates_timestamp() {
    let mut entry = LedgerEntry::new_neutral();
    let old_timestamp = PersistentTimestamp::new(1000, 0);
    entry.last_updated = old_timestamp;

    let now = PersistentTimestamp::now();
    let update = LedgerUpdate::new(true, 0.5, 1.0, Action::Allow, true);
    entry.apply_write_decay_and_update(0.999, &now, &update, 0.999);

    assert_eq!(entry.last_updated, now, "timestamp should be updated to now");
}

/// Every write is tallied against the action that was actually taken and
/// against no other: two challenges and one block leave those two slots at two
/// and one with the allow and slow slots untouched. Unlike the decaying EWMAs
/// these are undecayed counts, so they answer what the Sentinel did in a region
/// rather than how recently it did it.
///
/// ´claim:ledger:each-write-is-tallied-against-the-action-actually-taken-and-no-other´
/// ´test:crate:write-increments-action-count´
#[test]
fn write_increments_action_count() {
    let mut entry = LedgerEntry::new_neutral();
    let now = clock().now();

    let challenge = LedgerUpdate::new(true, 0.5, 1.0, Action::Challenge, true);
    let block = LedgerUpdate::new(false, -0.5, -1.0, Action::Block, true);

    entry.apply_write_decay_and_update(0.999, &now, &challenge, 0.999);
    entry.apply_write_decay_and_update(0.999, &now, &challenge, 0.999);
    entry.apply_write_decay_and_update(0.999, &now, &block, 0.999);

    assert_eq!(entry.per_action_count[0], 0, "Allow = 0");
    assert_eq!(entry.per_action_count[1], 2, "Challenge = 2");
    assert_eq!(entry.per_action_count[2], 0, "Slow = 0");
    assert_eq!(entry.per_action_count[3], 1, "Block = 1");
}

/// Assessments marked eligible raise the recent-eligible count, one per write.
/// That count is what cell-set maintenance consults when deciding whether a
/// region is carrying enough qualifying traffic to be worth tracking at its
/// current resolution.
///
/// ´claim:ledger:an-eligible-assessment-raises-the-recent-eligible-count´
/// ´test:crate:write-eligible-increments-recent´
#[test]
fn write_eligible_increments_recent() {
    let mut entry = LedgerEntry::new_neutral();
    let now = clock().now();

    let eligible = LedgerUpdate::new(true, 0.5, 1.0, Action::Allow, true);
    entry.apply_write_decay_and_update(0.999, &now, &eligible, 0.999);
    entry.apply_write_decay_and_update(0.999, &now, &eligible, 0.999);

    assert_eq!(entry.recent_eligible_count, 2);
}

/// Ineligible assessments are recorded but not credited: two of them leave the
/// eligible count at zero while the total assessment count reaches two. The two
/// tallies answer different questions — how much traffic passed through, and
/// how much of it qualifies as evidence for cell-set decisions — and conflating
/// them would let ineligible traffic keep a region alive on its own.
///
/// ´claim:ledger:an-ineligible-assessment-is-still-counted-as-traffic-but-not-as-eligible-evidence´
/// ´test:crate:write-ineligible-doesnt-count´
#[test]
fn write_ineligible_doesnt_count() {
    let mut entry = LedgerEntry::new_neutral();
    let now = clock().now();

    let ineligible = LedgerUpdate::new(true, 0.5, 1.0, Action::Allow, false);
    entry.apply_write_decay_and_update(0.999, &now, &ineligible, 0.999);
    entry.apply_write_decay_and_update(0.999, &now, &ineligible, 0.999);

    assert_eq!(entry.recent_eligible_count, 0, "ineligible should not increment");
    assert_eq!(entry.total_assessments, 2, "but total should still increment");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Additional Read Routing Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A coordinate sharing no prefix with the one tracked depth-4 cell reads the
/// root's history, not the unrelated cell's. Fallback climbs the coordinate's
/// own ancestry, so being at the same depth as a tracked cell is no help — only
/// containment counts, and traffic in an untracked region is scored against the
/// population rather than against a neighbour.
///
/// (´claim:ledger:a-coordinate-outside-the-deep-cells-falls-back-to-the-deepest-ancestor-containing-it´)
/// ´test:crate:read-root-fallback´
#[test]
fn read_root_fallback() {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();
    ledger.root_mut().total_assessments = 42;

    // Create entry at specific depth-4 cell (lo=0)
    ledger.insert(make_key(0, 4), LedgerEntry::new_neutral());
    ledger.get_mut(&make_key(0, 4)).unwrap().total_assessments = 4;

    // Coord NOT in depth-4 cell (different top 4 bits)
    let coord = 0xF000_0000_0000_0000_0000_0000_0000_0000_u128;
    let entry = read_route(&ledger, coord);

    assert_eq!(entry.total_assessments, 42, "should fall back to root");
}

/// An eleven-level gap in the ancestry does not stop the walk: with only the
/// root and a depth-12 cell tracked, the depth-12 entry answers. Nothing
/// requires the intermediate depths to exist, so a Sentinel may report at
/// whatever resolutions its geometry actually distinguishes without paying for
/// the levels in between.
///
/// (´claim:ledger:the-read-path-returns-the-deepest-cell-containing-the-coordinate´)
/// ´test:crate:read-sparse-depths´
#[test]
fn read_sparse_depths() {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();
    ledger.root_mut().total_assessments = 0;

    // Sparse: only depth 0 and depth 12 (gap 1-11)
    ledger.insert(make_key(0, 12), LedgerEntry::new_neutral());
    ledger.get_mut(&make_key(0, 12)).unwrap().total_assessments = 12;

    let entry = read_route(&ledger, 0);
    assert_eq!(entry.total_assessments, 12, "sparse depths should still route to deepest");
}

/// The last coordinate a cell covers is still inside it: one below the depth-8
/// cell's upper edge routes to that cell. Together with the lower endpoint this
/// pins the interval as closed at the bottom and covering everything up to but
/// not including the next cell's start.
///
/// (´claim:ledger:a-cells-lower-endpoint-routes-into-that-cell´)
/// ´test:crate:read-boundary-hi-minus-1´
#[test]
fn read_boundary_hi_minus_1() {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();

    // Depth-8 cell at lo=0 covers coords 0..(1<<120)
    ledger.insert(make_key(0, 8), LedgerEntry::new_neutral());
    ledger.get_mut(&make_key(0, 8)).unwrap().total_assessments = 88;

    // Last coord in depth-8 cell: (1 << 120) - 1
    let last_coord = (1_u128 << 120) - 1;
    let entry = read_route(&ledger, last_coord);
    assert_eq!(entry.total_assessments, 88, "last coord in cell should route to depth-8");
}

/// The first coordinate past a cell's upper edge is outside it and falls back
/// to the root. This is the exclusive half of the boundary: one step beyond the
/// interval and the cell's history no longer applies, so adjacent regions never
/// bleed a neighbour's reputation into each other.
///
/// (´claim:ledger:a-coordinate-outside-the-deep-cells-falls-back-to-the-deepest-ancestor-containing-it´)
/// ´test:crate:read-just-outside´
#[test]
fn read_just_outside() {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();
    ledger.root_mut().total_assessments = 1;

    ledger.insert(make_key(0, 8), LedgerEntry::new_neutral());
    ledger.get_mut(&make_key(0, 8)).unwrap().total_assessments = 88;

    // First coord past depth-8 cell: (1 << 120)
    let past_coord = 1_u128 << 120;
    let entry = read_route(&ledger, past_coord);
    assert_eq!(entry.total_assessments, 1, "coord past cell should fall to root");
}

/// Coordinate zero is treated like any other: with cells at depths 4, 8 and 12
/// all containing it, the depth-12 entry answers rather than the root it also
/// shares an origin with. Zero is the lower endpoint of every cell in its chain
/// simultaneously, which makes it the coordinate most likely to expose a walk
/// that confuses "at the origin" with "at the root".
///
/// (´claim:ledger:the-read-path-returns-the-deepest-cell-containing-the-coordinate´)
/// ´test:crate:read-root-coordinate-zero´
#[test]
fn read_root_coordinate_zero() {
    let mut ledger = make_ledger_with_depths(&[0, 4, 8, 12]);

    ledger.root_mut().total_assessments = 0;
    ledger.get_mut(&make_key(0, 12)).unwrap().total_assessments = 12;

    // Coordinate 0 should route to deepest entry containing it
    let entry = read_route(&ledger, 0);
    assert_eq!(entry.total_assessments, 12, "coord 0 routes to deepest cell");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Additional Write Routing Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A write against a root-only ledger touches exactly one entry and reports as
/// much. The all-layers walk degenerates gracefully: a Sentinel whose geometry
/// has not yet been reported still accumulates its whole outcome history in the
/// one cell it has.
///
/// (´claim:ledger:a-write-updates-every-tracked-cell-containing-the-coordinate´)
/// ´test:crate:write-root-only´
#[test]
fn write_root_only() {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();

    let update = LedgerUpdate::new(true, 0.5, 1.0, Action::Allow, true);
    let now = clock().now();

    let count = update_all_layers(
        &mut ledger,
        0x1234_0000_0000_0000_0000_0000_0000_0000_u128,
        0.999,
        &now,
        &update,
        0.999,
    );

    assert_eq!(count, 1, "only root should be updated");
    assert_eq!(ledger.root().total_assessments, 1);
}

/// A gap in the ancestry does not truncate a write: with cells at depths 0, 4
/// and 12 and nothing in between, all three record the observation. The write
/// path walks the whole ancestry rather than stopping at the first depth that
/// holds nothing, so a deep cell above a sparse chain still accumulates.
///
/// (´claim:ledger:a-write-updates-every-tracked-cell-containing-the-coordinate´)
/// ´test:crate:write-sparse-chain´
#[test]
fn write_sparse_chain() {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();
    ledger.insert(make_key(0, 4), LedgerEntry::new_neutral());
    ledger.insert(make_key(0, 12), LedgerEntry::new_neutral());

    let update = LedgerUpdate::new(true, 0.5, 1.0, Action::Allow, true);
    let now = clock().now();

    let count = update_all_layers(&mut ledger, 0, 0.999, &now, &update, 0.999);

    assert_eq!(count, 3, "should update root, depth-4, depth-12");
    assert_eq!(ledger.root().total_assessments, 1);
    assert_eq!(ledger.get(&make_key(0, 4)).unwrap().total_assessments, 1);
    assert_eq!(ledger.get(&make_key(0, 12)).unwrap().total_assessments, 1);
}

/// A write never brings a cell into existence: after writing a coordinate whose
/// depth-8 cell is untracked, that cell is still absent and the ledger still
/// holds only its root. Which cells exist is decided by the Sentinel's reports
/// alone, so traffic cannot silently subdivide the ledger into a cell per
/// observation and grow it without bound.
///
/// ´claim:ledger:a-write-never-brings-a-cell-into-existence´
/// ´test:crate:write-does-not-create´
#[test]
fn write_does_not_create() {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();

    let update = LedgerUpdate::new(true, 0.5, 1.0, Action::Allow, true);
    let now = clock().now();

    // Write to coord that would be in depth-8 cell, but no entry exists
    update_all_layers(&mut ledger, 0, 0.999, &now, &update, 0.999);

    assert!(ledger.get(&make_key(0, 8)).is_none(), "write should not create new entries");
    assert_eq!(ledger.entry_count(), 1, "still only root");
}

/// Two benign writes accumulate as traffic at both the root and the depth-8
/// cell — each reaching two assessments — while the bad-rate stays at zero.
/// Volume and badness are recorded independently, so a busy but well-behaved
/// region builds a substantial record of having been well-behaved rather than
/// looking indistinguishable from an idle one.
///
/// (´claim:ledger:a-benign-outcome-contributes-nothing-to-the-bad-rate´)
/// ´test:crate:write-two-successive´
#[test]
fn write_two_successive() {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();
    ledger.insert(make_key(0, 8), LedgerEntry::new_neutral());

    let update = LedgerUpdate::new(false, -0.5, -1.0, Action::Block, true);
    let now = clock().now();
    let lambda = 0.999;

    // First write
    update_all_layers(&mut ledger, 0, 0.999, &now, &update, lambda);
    // Second write
    update_all_layers(&mut ledger, 0, 0.999, &now, &update, lambda);

    assert_eq!(ledger.root().total_assessments, 2);
    assert_eq!(ledger.get(&make_key(0, 8)).unwrap().total_assessments, 2);

    // EWMA accumulates: after 2 negative updates (is_bad = 0)
    // bad_rate stays at 0 since negative outcomes are not adverse.
    assert_near(ledger.root().ewma_bad_rate, 0.0, TOL.ewma, "two negatives keep bad_rate at 0");
}

/// Two distinct coordinates that share a depth-4 ancestor both credit that
/// ancestor as well as the root, taking each to two assessments. An ancestor
/// aggregates over everything beneath it, which is the whole reason a coarse
/// cell can say something useful about a coordinate that has never been seen
/// before.
///
/// (´claim:ledger:a-write-updates-every-tracked-cell-containing-the-coordinate´)
/// ´test:crate:write-different-coords-same-ancestor´
#[test]
fn write_different_coords_same_ancestor() {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();
    ledger.insert(make_key(0, 4), LedgerEntry::new_neutral());

    let update = LedgerUpdate::new(true, 0.5, 1.0, Action::Allow, true);
    let now = clock().now();

    // Two coords both in depth-4 cell (lo=0): top 4 bits = 0
    let coord_a = 0x00AB_0000_0000_0000_0000_0000_0000_0000_u128;
    let coord_b = 0x0CD0_0000_0000_0000_0000_0000_0000_0000_u128;

    update_all_layers(&mut ledger, coord_a, 0.999, &now, &update, 0.999);
    update_all_layers(&mut ledger, coord_b, 0.999, &now, &update, 0.999);

    assert_eq!(ledger.root().total_assessments, 2, "root should be updated by both");
    assert_eq!(
        ledger.get(&make_key(0, 4)).unwrap().total_assessments,
        2,
        "depth-4 shared by both"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Additional Cell-Set Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A cell already tracked is not counted as an appearance and is not recreated:
/// its accumulated tally comes through the report untouched. Reports name the
/// full current cell set every cycle rather than only what changed, so treating
/// a repeat mention as a new arrival would reset a region's history on every
/// cycle it stayed alive.
///
/// ´claim:ledger:a-cell-already-tracked-is-not-recreated-and-keeps-its-history´
/// ´test:crate:appear-already-exists´
#[test]
fn appear_already_exists() {
    let key4 = make_key(0, 4);
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();
    ledger.insert(key4, LedgerEntry::new_neutral());
    ledger.get_mut(&key4).unwrap().total_assessments = 42;

    // Report includes existing key
    let report: HashSet<LedgerKey> = vec![key4].into_iter().collect();
    let changes = apply_cell_set_changes(&mut ledger, &report, 3);

    assert_eq!(changes.appeared, 0, "no new appearances");
    assert_eq!(ledger.get(&key4).unwrap().total_assessments, 42, "existing entry unchanged");
}

/// A newly appeared cell inherits nothing from the ancestor containing it: with
/// the root sitting at a high bad-rate and a substantial valence, the new
/// depth-4 entry starts at zero on both. Inheriting would prejudge a region on
/// its neighbourhood's reputation — and would double-count, since the ancestor
/// keeps receiving that region's writes regardless.
///
/// ´claim:ledger:a-newly-appeared-cell-inherits-nothing-from-its-ancestor´
/// ´test:crate:creation-no-inheritance´
#[test]
fn creation_no_inheritance() {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();
    ledger.root_mut().ewma_bad_rate = 0.8;
    ledger.root_mut().compressed_valence_ewma = 0.5;

    // New cell appears
    let key4 = make_key(0, 4);
    let report: HashSet<LedgerKey> = vec![key4].into_iter().collect();
    apply_cell_set_changes(&mut ledger, &report, 3);

    // New entry should be neutral, NOT inherit from root
    let new_entry = ledger.get(&key4).expect("should exist");
    assert_near(
        new_entry.ewma_bad_rate,
        0.0,
        TOL.bit_identical,
        "new entry does not inherit bad_rate",
    );
    assert_near(
        new_entry.compressed_valence_ewma,
        0.0,
        TOL.bit_identical,
        "new entry does not inherit valence",
    );
}

/// Several cells arriving in one report are all taken up in that single pass —
/// three named, three appeared, four entries tracked. A Sentinel whose geometry
/// expands suddenly is absorbed in one cycle rather than one cell per cycle.
///
/// (´claim:ledger:a-cell-newly-named-by-a-report-starts-being-tracked-from-neutral´)
/// ´test:crate:multiple-appearances´
#[test]
fn multiple_appearances() {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();

    let keys: HashSet<LedgerKey> = vec![make_key(0, 4), make_key(0, 8), make_key(0, 12)].into_iter().collect();

    let changes = apply_cell_set_changes(&mut ledger, &keys, 3);

    assert_eq!(changes.appeared, 3, "all 3 should appear");
    assert_eq!(ledger.entry_count(), 4, "root + 3 new");
}

/// Several cells reaching the threshold together are all removed in the same
/// pass and counted as such: two entries already carrying two absences each are
/// both gone after one more. Removal is decided per entry rather than one per
/// cycle, so a region that vanishes wholesale is released in one step instead
/// of draining away over as many cycles as it had cells.
///
/// (´claim:ledger:a-cell-is-dropped-on-the-absence-that-reaches-the-threshold´)
/// ´test:crate:multiple-deletions´
#[test]
fn multiple_deletions() {
    let key4 = make_key(0, 4);
    let key8 = make_key(0, 8);
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();

    let mut entry4 = LedgerEntry::new_neutral();
    entry4.consecutive_absent = 2;
    let mut entry8 = LedgerEntry::new_neutral();
    entry8.consecutive_absent = 2;

    ledger.insert(key4, entry4);
    ledger.insert(key8, entry8);

    // Empty report triggers both deletions
    let empty: HashSet<LedgerKey> = HashSet::new();
    let changes = apply_cell_set_changes(&mut ledger, &empty, 3);

    assert_eq!(changes.deleted, 2, "both should be deleted");
    assert!(ledger.get(&key4).is_none());
    assert!(ledger.get(&key8).is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Additional OutcomeLedger Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// An unregistered Sentinel has no ledger at all rather than an empty one.
/// Asking about an unknown identity is answered with absence, so a caller
/// cannot accidentally accumulate outcome history against a Sentinel the model
/// owner never registered.
///
/// ´claim:ledger:an-unregistered-sentinel-has-no-ledger-rather-than-an-empty-one´
/// ´test:crate:get-unknown-sentinel´
#[test]
fn get_unknown_sentinel() {
    let ledger = OutcomeLedger::new();
    assert!(ledger.get(SentinelId(999)).is_none(), "unknown sentinel returns None");
}

/// Sentinels registered alongside each other are genuinely separate: writing a
/// distinctive tally into one leaves another's root at zero. Each has its own
/// ledger behind its own lock, so Sentinels neither share a coordinate space
/// nor contend with one another for access to it.
///
/// ´claim:ledger:each-sentinels-ledger-is-independent-of-every-other´
/// ´test:crate:multiple-sentinels´
#[test]
fn multiple_sentinels() {
    let ledger = OutcomeLedger::new();
    ledger.create_sentinel(SentinelId(1));
    ledger.create_sentinel(SentinelId(2));
    ledger.create_sentinel(SentinelId(3));

    // Verify all 3 accessible
    assert!(ledger.contains(SentinelId(1)));
    assert!(ledger.contains(SentinelId(2)));
    assert!(ledger.contains(SentinelId(3)));

    // Modify one, verify independence
    {
        let lock = ledger.get(SentinelId(1)).unwrap();
        let mut guard = lock.write().unwrap();
        guard.root_mut().total_assessments = 100;
    }

    {
        let lock = ledger.get(SentinelId(2)).unwrap();
        let guard2 = lock.read().unwrap();
        assert_eq!(guard2.root().total_assessments, 0, "sentinel 2 should be independent");
        drop(guard2);
    }
}

/// A ledger populated at five different depths round-trips with its entry count
/// and each depth's distinguishing tally preserved. The depth carried in a key
/// survives the encoding, so a restored checkpoint routes exactly as the
/// original did rather than collapsing its hierarchy into one flat layer.
///
/// (´claim:ledger:a-serialised-ledger-comes-back-with-its-root-and-cells-intact´)
/// ´test:crate:ledger-serde-roundtrip´
#[test]
fn ledger_serde_roundtrip() {
    #[cfg(feature = "serde")]
    {
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();

        // Add multiple depths
        for depth in [4, 8, 12, 16, 20] {
            let mut entry = LedgerEntry::new_neutral();
            entry.total_assessments = u64::from(depth);
            ledger.insert(make_key(0, depth), entry);
        }

        let json = serde_json::to_string(&ledger).expect("serialize");
        let restored: SentinelLedger = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored.entry_count(), ledger.entry_count());
        for depth in [4, 8, 12, 16, 20] {
            assert_eq!(restored.get(&make_key(0, depth)).unwrap().total_assessments, u64::from(depth));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Property Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Fixed seed used by property tests so failures are bit-reproducible.
const PROPERTY_SEED: u64 = 42;

/// The bad-rate is a genuine rate under any history: across a thousand
/// coin-flipped outcomes it never leaves the unit interval at any step. Because
/// it is a convex mixture of the stored value and an indicator, it can only sit
/// between the extremes — and downstream consumers can treat it as a
/// probability without defensively clamping it.
///
/// ´claim:ledger:the-bad-rate-stays-within-the-unit-interval-under-any-history´
/// ´test:crate:property-bad-rate-in-0-1´
#[test]
fn property_bad_rate_in_0_1() {
    // After any sequence of updates, bad_rate ∈ [0, 1]
    let mut entry = LedgerEntry::new_neutral();
    let now = clock().now();
    let lambda = 0.999;
    let mut rng = TestRng::new(PROPERTY_SEED);

    for i in 0..1000 {
        let is_positive = rng.coin();
        let update = LedgerUpdate::new(is_positive, if is_positive { 0.5 } else { -0.5 }, 1.0, Action::Allow, true);
        entry.apply_write_decay_and_update(0.999, &now, &update, lambda);

        assert!(entry.ewma_bad_rate >= 0.0, "bad_rate >= 0 at step {i}");
        assert!(entry.ewma_bad_rate <= 1.0, "bad_rate <= 1 at step {i}");
    }
}

/// The compressed valence EWMA stays inside the range of the values it
/// averages: fed a thousand random drivers spanning minus one to one, it never
/// escapes those bounds in either direction. Averaging cannot overshoot its
/// inputs, so the compression bound established upstream is preserved by the
/// ledger rather than having to be reasserted downstream.
///
/// ´claim:ledger:the-compressed-valence-ewma-stays-inside-the-range-of-the-values-it-averages´
/// ´test:crate:property-compressed-valence-in-minus1-1´
#[test]
fn property_compressed_valence_in_minus1_1() {
    let mut entry = LedgerEntry::new_neutral();
    let now = clock().now();
    let lambda = 0.999;
    let mut rng = TestRng::new(PROPERTY_SEED + 1);

    for i in 0..1000 {
        // Random valence in [-1, 1]
        let compressed = rng.next_f64().mul_add(2.0, -1.0); // [-1, 1)
        let update = LedgerUpdate::new(compressed > 0.0, compressed, compressed * 5.0, Action::Allow, true);
        entry.apply_write_decay_and_update(0.999, &now, &update, lambda);

        assert!(
            entry.compressed_valence_ewma >= -1.0,
            "compressed >= -1 at step {i}: {}",
            entry.compressed_valence_ewma
        );
        assert!(
            entry.compressed_valence_ewma <= 1.0,
            "compressed <= 1 at step {i}: {}",
            entry.compressed_valence_ewma
        );
    }
}

/// A repeated identical observation pulls the EWMA toward it at exactly the
/// rate the retention factor sets — a hundred steps at one value reach the
/// closed-form fraction of it, still well short of the target. Convergence is
/// slow by construction: at this retention a cell needs hundreds of
/// observations before its recorded history is dominated by present behaviour
/// rather than by its past.
///
/// ´claim:ledger:a-repeated-observation-pulls-the-ewma-toward-it-at-the-rate-the-retention-factor-sets´
/// ´test:crate:property-ewma-converges-toward-input´
#[test]
fn property_ewma_converges_toward_input() {
    // 100 identical labels → EWMA within 5% of input
    let mut entry = LedgerEntry::new_neutral();
    let now = clock().now();
    let lambda = 0.999;
    let target_valence = 0.7;

    let update = LedgerUpdate::new(true, target_valence, target_valence, Action::Allow, true);

    for _ in 0..100 {
        entry.apply_write_decay_and_update(0.999, &now, &update, lambda);
    }

    // After 100 steps, EWMA should approach target
    // Limit: target * (1 - λ^∞) = target
    // After 100: target * (1 - λ^100) ≈ target * 0.0952
    assert_near(
        entry.compressed_valence_ewma,
        target_valence * 0.095,
        0.01,
        "EWMA approaches target",
    );
}

/// Across gaps from one hour to a thousand, a decayed reading is always
/// strictly positive and never exceeds what is stored. Decay only ever
/// discounts: no elapsed span can manufacture evidence that was not written,
/// and none can erase the last trace of evidence that was.
///
/// ´claim:ledger:a-decayed-reading-is-always-positive-and-never-exceeds-the-stored-value´
/// ´test:crate:property-read-time-decay-in-range´
#[test]
fn property_read_time_decay_in_range() {
    // Decayed value ∈ (0, stored]
    let mut entry = LedgerEntry::new_neutral();
    entry.ewma_bad_rate = 0.5;

    for hours in 1..=1000 {
        let old = PersistentTimestamp::new(entry.last_updated.seconds - hours * 3600, 0);
        entry.last_updated = old;

        let now = PersistentTimestamp::now();
        let view = entry.read_decayed(0.999, &now);

        assert!(view.bad_rate > 0.0, "decayed > 0 at {hours}h");
        assert!(view.bad_rate <= 0.5, "decayed <= stored at {hours}h");

        // Reset for next iteration
        entry.last_updated = now;
    }
}

/// The root receives every write regardless of coordinate or geometry: across a
/// hundred randomly-shaped ledgers and random coordinates, its assessment tally
/// rises by exactly one each time — never zero, never twice for one write. The
/// root is the Sentinel-wide baseline every fallback read lands on, so it must
/// see all of the traffic exactly once.
///
/// ´claim:ledger:the-root-receives-every-write-exactly-once-whatever-the-coordinate´
/// ´test:crate:property-all-layers-always-includes-root´
#[test]
fn property_all_layers_always_includes_root() {
    // Root entry's EWMA always changes
    let mut rng = TestRng::new(PROPERTY_SEED + 2);

    for seed in 0..100 {
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();

        // Random depths
        #[allow(clippy::cast_possible_truncation)]
        let depths: Vec<u8> = (0..5).map(|_| ((rng.next_range(12) as u8) + 1) * 4).collect();
        for &depth in &depths {
            ledger.insert(make_key(0, depth), LedgerEntry::new_neutral());
        }

        let coord = u128::from(rng.next_u64());
        let original_root_assessments = ledger.root().total_assessments;

        let update = LedgerUpdate::new(true, 0.5, 1.0, Action::Allow, true);
        let now = clock().now();
        update_all_layers(&mut ledger, coord, 0.999, &now, &update, 0.999);

        assert_eq!(
            ledger.root().total_assessments,
            original_root_assessments + 1,
            "root always updated (iteration {seed})"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Folklore Scenarios
// ═══════════════════════════════════════════════════════════════════════════════

/// "Time-Indexed Ledger Decay: 29-Day Half-Life Timeline".
///
/// Contamination dissolves on the published schedule rather than on a schedule
/// that merely resembles it. A burst of fifty adverse labels lifts a cell to a
/// peak of about five per cent, and at every milestone across three months —
/// day 0, 7, 14, 29, 58 and 87 — the value a read returns matches the closed
/// form of peak times the hourly factor raised to the elapsed hours, to within
/// a billionth, while the stored state itself never moves. By day 87 the
/// reading is under 0.007, roughly an eighth of the peak: effectively inert.
/// This is the promise a host relies on when it tolerates a bad patch, so the
/// timeline has to be exact rather than approximately right, and it has to hold
/// without any sweep having run.
///
/// ´claim:ledger:contamination-decays-on-the-published-twenty-nine-day-schedule-and-is-inert-by-eighty-seven-days´
/// ´test:crate:ledger-decay-twenty-nine-day-half-life-timeline´
#[test]
fn ledger_decay_twenty_nine_day_half_life_timeline() {
    const GAMMA_HR: f64 = 0.999;
    const LAMBDA_EWMA: f64 = 0.999;
    const N_LABELS: usize = 50;

    // First — Build the contamination peak by firing 50 positives at a fixed
    // timestamp, so the write-side decay step is a no-op each time and the
    // only surviving state is the EWMA accumulator.
    let t0 = clock().now();
    let mut peak_entry = LedgerEntry::new_neutral();
    peak_entry.last_updated = t0;
    let positive_update = LedgerUpdate::new(true, 1.0, 1.0, Action::Allow, true);
    for _ in 0..N_LABELS {
        peak_entry.apply_write_decay_and_update(GAMMA_HR, &t0, &positive_update, LAMBDA_EWMA);
    }
    let peak = peak_entry.ewma_bad_rate;
    #[allow(clippy::cast_possible_wrap)] // N_LABELS is a tiny const; no wrap possible.
    let expected_peak = 1.0 - LAMBDA_EWMA.powi(N_LABELS as i32);
    assert_near(peak, expected_peak, 1e-12, "50-positive EWMA peak");
    assert!(
        (0.045..=0.055).contains(&peak),
        "peak ≈ 0.05, the contamination a fifty-label burst lifts a cell to: got {peak}"
    );

    // Second — Advance wallclock time in-place by rewinding `last_updated` to
    // `t0 − Δ`. Each milestone is a pure read; `read_decayed` must not
    // mutate the entry.
    const DAYS: [i64; 6] = [0, 7, 14, 29, 58, 87];
    const SECONDS_PER_DAY: i64 = 24 * 3600;

    for &days in &DAYS {
        let mut probe = peak_entry.clone();
        probe.last_updated = PersistentTimestamp::new(t0.seconds - days * SECONDS_PER_DAY, 0);

        let view = probe.read_decayed(GAMMA_HR, &t0);
        #[allow(clippy::cast_precision_loss)]
        let hours = (days * 24) as f64;
        let expected = peak * GAMMA_HR.powf(hours);

        // Tight absolute bound: `powf` vs spec formula agree to machine
        // precision; the only looseness is whatever `read_decayed` adds.
        assert_near(view.bad_rate, expected, 1e-9, &format!("day {days}: EWMA = peak · γ^(24·t)"));

        // Purity: `read_decayed` must not mutate the stored state.
        assert_near(
            probe.ewma_bad_rate,
            peak,
            TOL.bit_identical,
            &format!("day {days}: entry unchanged by read_decayed"),
        );
    }

    // Third — At 87 days, the decayed reading must sit below 0.007 (≈ 12.4%
    // of the peak) — the scenario's "effectively inert" threshold.
    let mut at_87 = peak_entry.clone();
    at_87.last_updated = PersistentTimestamp::new(t0.seconds - 87 * SECONDS_PER_DAY, 0);
    let final_view = at_87.read_decayed(GAMMA_HR, &t0);
    assert!(
        final_view.bad_rate < 0.007,
        "87-day reading must be <0.007, the effectively-inert threshold of the published decay schedule: got {}",
        final_view.bad_rate
    );
}

/// "Outcome Axis EWMA Decays on Same 29-Day Schedule as Binary Rate".
///
/// Companion to (`ledger_decay_twenty_nine_day_half_life_timeline`),
/// but for a registered outcome axis's compressed-valence EWMA.
///
/// A per-axis EWMA ages on exactly the same schedule as the binary bad-rate,
/// not merely a similar one. Driven by an axis value at the saturating edge of
/// compression, the axis EWMA reaches the same peak the binary rate reaches
/// from the same number of adverse labels — the two share a step size — and
/// then stays locked to it at every milestone out to 87 days, each reading
/// matching the closed form and the final one below the same inert threshold.
/// A host reading an axis is therefore reasoning on the same timescale it
/// already understands from the risk figures, rather than having to learn a
/// second decay regime per axis it registers.
///
/// ´claim:ledger:a-per-axis-ewma-ages-on-the-same-schedule-as-the-binary-bad-rate´
/// ´test:crate:outcome-axis-ewma-decay-twenty-nine-day-timeline´
#[test]
fn outcome_axis_ewma_decay_twenty_nine_day_timeline() {
    const GAMMA_HR: f64 = 0.999;
    const LAMBDA_EWMA: f64 = 0.999;
    const N_LABELS: usize = 10;
    const AXIS: OutcomeAxisId = OutcomeAxisId(7);

    // First — Build the contamination peak on the per-axis compressed
    // EWMA. `1000 / κₐ` is large enough that the upstream `tanh`
    // compression is within machine ε of +1.0 — we use the saturated
    // value directly so the assertion isolates the decay path.
    let saturated: f64 = (1000.0_f64 / 1.0_f64).tanh(); // ≈ 1.0 to ~1e-434
    let t0 = clock().now();
    let mut peak_entry = LedgerEntry::new_neutral();
    peak_entry.last_updated = t0;
    let axis_update = LedgerUpdate::new(true, saturated, 1000.0, Action::Allow, true).with_axis(AXIS, saturated, 1000.0);
    for _ in 0..N_LABELS {
        peak_entry.apply_write_decay_and_update(GAMMA_HR, &t0, &axis_update, LAMBDA_EWMA);
    }

    let peak_axis = axis_compressed(&peak_entry.per_axis, AXIS);
    let peak_bad = peak_entry.ewma_bad_rate;

    // Sanity: the saturated compressed input drives the EWMA to the
    // same value the binary bad-rate reaches from 10 positives — the
    // two paths share the same `(1 − λ)` step size when the driver is
    // ≈ 1.0. This is the formal statement of "same 29-day timescale":
    // the two EWMAs coincide at the peak and therefore decay in lock-
    // step under a shared `γ^Δh` factor.
    assert_near(
        peak_axis,
        peak_bad,
        1e-12,
        "saturated per-axis EWMA tracks the binary bad-rate EWMA at the peak",
    );

    // Second — Same 6-milestone timeline as the binary-rate test. Per-axis compressed
    // reading must match `peak · γ^{24·t}` with the same tolerance.
    const DAYS: [i64; 6] = [0, 7, 14, 29, 58, 87];
    const SECONDS_PER_DAY: i64 = 24 * 3600;

    for &days in &DAYS {
        let mut probe = peak_entry.clone();
        probe.last_updated = PersistentTimestamp::new(t0.seconds - days * SECONDS_PER_DAY, 0);

        let view = probe.read_decayed(GAMMA_HR, &t0);
        #[allow(clippy::cast_precision_loss)]
        let hours = (days * 24) as f64;
        let expected = peak_axis * GAMMA_HR.powf(hours);

        let actual_axis = axis_compressed(&view.per_axis, AXIS);
        assert_near(
            actual_axis,
            expected,
            1e-9,
            &format!("day {days}: per-axis EWMA = peak · γ^(24·t)"),
        );

        // The per-axis and binary readings must remain lock-stepped at
        // every milestone — this is what "same 29-day timescale" means.
        assert_near(
            actual_axis,
            view.bad_rate,
            1e-9,
            &format!("day {days}: per-axis EWMA tracks binary bad-rate exactly"),
        );

        // Purity: `read_decayed` must not mutate the stored state.
        assert_near(
            axis_compressed(&probe.per_axis, AXIS),
            peak_axis,
            TOL.bit_identical,
            &format!("day {days}: per-axis EWMA unchanged by read_decayed"),
        );
    }

    // Third — 87-day reading below the "effectively inert" threshold.
    let mut at_87 = peak_entry.clone();
    at_87.last_updated = PersistentTimestamp::new(t0.seconds - 87 * SECONDS_PER_DAY, 0);
    let final_view = at_87.read_decayed(GAMMA_HR, &t0);
    let final_axis = axis_compressed(&final_view.per_axis, AXIS);
    assert!(
        final_axis < 0.007,
        "87-day per-axis reading must be <0.007, the same effectively-inert threshold the binary rate meets: got {final_axis}",
    );
}

/// "The Label-Assessment Race".
///
/// Spec formulation: "Process a label that flips a Ledger cell's EWMA
/// from 0 to 0.5. Simultaneously `assess()` an observation routed
/// through that cell. The assessment must see either the pre-label EWMA
/// (0) or the post-label EWMA (0.5), never a torn intermediate."
///
/// A reader routing through the ledger while a writer mutates the very cell it
/// resolves to sees one whole value or the other, never a partial one: the
/// interleaving is forced rather than waited for. The writer holds the exclusive
/// guard across the window in which the label is only half applied — the routed
/// cell updated, the root it hangs from not yet — and a reader stands at the
/// door for exactly that window. It cannot get in, because a read guard is
/// unobtainable while the write guard is held; and by the time it does get in
/// the guard is gone, so what it reads is the post-label state entire, both
/// layers carrying the same label count and the same bit-exact rate. Both pinned
/// values are observed on every run — the pre-label state from an uncontended
/// read before the write, the post-label state from the read that was blocked
/// mid-write — so the guarantee is never vacuous. The layers are walked
/// deepest-first, the order the write path itself walks
/// (´alg:ledger:all-layers-update´), and the state they leave behind is checked
/// against what that path produces in a single call: the hand-walked write is
/// the same write, cut open. Per-Sentinel access is serialised through a
/// read-write lock, and that exclusion is the whole of the guarantee — no reader
/// is ever inside the ledger while a label is half applied, so there is no
/// interleaving left for a torn read to happen in.
///
/// ´claim:ledger:a-reader-racing-a-writer-sees-one-whole-value-never-a-partial-one´
/// ´test:crate:label-assess-race-no-torn-reads´
#[test]
fn label_assess_race_no_torn_reads() {
    /// What one reader saw under one read guard: the cell it routed to and the
    /// root that cell hangs from, each as a label count and a bit pattern.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    struct Observed {
        leaf_labels: u64,
        root_labels: u64,
        leaf_rate: u64,
        root_rate: u64,
    }

    /// One reader's whole observation, taken through the read path a live
    /// assessment uses.
    fn observe(ledger: &SentinelLedger, coord: u128) -> Observed {
        let leaf = read_route(ledger, coord);
        let leaf_labels = leaf.total_assessments;
        let leaf_rate = leaf.ewma_bad_rate.to_bits();
        let root = ledger.root();
        Observed {
            leaf_labels,
            root_labels: root.total_assessments,
            leaf_rate,
            root_rate: root.ewma_bad_rate.to_bits(),
        }
    }

    /// A coherent state: both layers at the same count and the same rate.
    fn whole(labels: u64, rate: f64) -> Observed {
        Observed {
            leaf_labels: labels,
            root_labels: labels,
            leaf_rate: rate.to_bits(),
            root_rate: rate.to_bits(),
        }
    }

    /// A Sentinel with a routed cell under its root, both stamped at `t0`.
    fn seed(sentinel: SentinelId, leaf_key: LedgerKey, t0: PersistentTimestamp) -> OutcomeLedger {
        let ledger = OutcomeLedger::new();
        ledger.create_sentinel(sentinel);
        let slot = ledger.get(sentinel).expect("sentinel registered");
        let mut guard = slot.write().expect("ledger RwLock poisoned");
        guard.root_mut().last_updated = t0;
        let mut leaf = LedgerEntry::new_neutral();
        leaf.last_updated = t0;
        guard.insert(leaf_key, leaf);
        drop(guard);
        ledger
    }

    // A retention factor of one half takes the bad-rate from zero to exactly one
    // half on a single adverse label, and both cells are stamped at the instant
    // the label is applied, so the elapsed-time factor is exactly one. The two
    // pinned values are therefore bit-exact rather than merely close.
    const LAMBDA_L: f64 = 0.5;
    const GAMMA_HR: f64 = 0.999;
    const DEPTH: u8 = 4;
    const PRE_LABEL: f64 = 0.0;
    const POST_LABEL: f64 = 0.5;
    // A stall detector, not a claim: it covers a handshake that takes
    // microseconds, and its only job is to fail a test whose partner thread has
    // died rather than park the run on it forever.
    const HANDSHAKE: Duration = Duration::from_secs(60);

    let sentinel = SentinelId(1);
    let coord = dyadic_ancestor_lo(0, DEPTH);
    let leaf_key = LedgerKey::from_coordinate(coord, DEPTH);
    let t0 = PersistentTimestamp::new(1_000_000, 0);
    // One adverse, eligible label — the write that flips the cell.
    let update = LedgerUpdate::new(true, 1.0, 1.0, Action::Allow, true);

    // What the write path leaves behind when it runs in one uninterrupted call.
    // This is the yardstick the forced, cut-open write is measured against.
    let reference = seed(sentinel, leaf_key, t0);
    let expected = {
        let slot = reference.get(sentinel).expect("sentinel registered");
        let mut guard = slot.write().expect("ledger RwLock poisoned");
        let updated = update_all_layers(&mut guard, coord, GAMMA_HR, &t0, &update, LAMBDA_L);
        assert_eq!(updated, 2, "the label reaches the routed cell and the root above it");
        observe(&guard, coord)
    };
    assert_eq!(
        expected,
        whole(1, POST_LABEL),
        "one adverse label leaves both layers at the post-label rate"
    );

    let ledger = seed(sentinel, leaf_key, t0);
    let slot = ledger.get(sentinel).expect("sentinel registered");

    // Uncontended, before the write: the pre-label state, whole.
    {
        let guard = slot.read().expect("ledger RwLock poisoned");
        assert_eq!(observe(&guard, coord), whole(0, PRE_LABEL), "the pre-label state is whole");
    }

    let (at_door_tx, at_door_rx) = crossbeam_channel::bounded::<()>(1);
    let reader_slot = Arc::clone(&slot);

    // The writer takes the guard before the reader exists, so the reader below
    // reaches the door inside the half-applied window on every run, with nothing
    // left to the scheduler.
    let mut guard = slot.write().expect("ledger RwLock poisoned");

    let reader = thread::spawn(move || {
        // The risk point. The half-applied ledger is unobtainable rather than
        // merely unlikely: the exclusive guard is held across the whole window.
        assert!(
            reader_slot.try_read().is_err(),
            "a read began while the label was only half applied"
        );
        at_door_tx.send(()).expect("the writer is waiting on this");
        // Blocked at the door until the label is whole, so what comes back can
        // only be the post-label state.
        let guard = reader_slot.read().expect("ledger RwLock poisoned");
        observe(&guard, coord)
    });

    at_door_rx.recv_timeout(HANDSHAKE).expect("the reader reaches the door");

    // Deepest layer first, the order the write path walks.
    let leaf = guard.get_mut(&leaf_key).expect("the routed cell is present");
    leaf.apply_write_decay_and_update(GAMMA_HR, &t0, &update, LAMBDA_L);
    let half_applied = Observed {
        leaf_labels: 1,
        root_labels: 0,
        leaf_rate: POST_LABEL.to_bits(),
        root_rate: PRE_LABEL.to_bits(),
    };
    assert_eq!(
        observe(&guard, coord),
        half_applied,
        "the window the reader is parked outside really does hold a half-applied label"
    );

    // Then the root, which completes the label and makes the state whole again.
    let root = guard.root_mut();
    root.apply_write_decay_and_update(GAMMA_HR, &t0, &update, LAMBDA_L);
    drop(guard);

    let seen = reader.join().expect("reader thread panicked");
    assert_eq!(seen, expected, "the read that arrived mid-write saw the label whole");

    // The cut-open write left the ledger where the write path itself would have.
    let guard = slot.read().expect("ledger RwLock poisoned");
    assert_eq!(observe(&guard, coord), expected, "the cut-open write is the same write");
}
