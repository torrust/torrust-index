// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`checkpoint_round_trip`] | persistence | A checkpoint written to disk and read back yields the state that was saved: the label sequence, the four calibration scalars and the shape and mean of each core model all come back as they went in. This is the whole premise of a warm restart — the file has to be a faithful account of the model, not an approximation of it. |
//! | [`checkpoint_restores_marginalisation_error_ledger`] | persistence | The marginalisation ledger crosses the file and rebuild boundary whole: every event class, the fallback and certificate counters, the accumulated discarded mass and the worst loss fraction return with their distinctive values. These counters describe the deployment rather than one process, so a restart must not make approximation already committed disappear from the health surface. |
//! | [`checkpoint_crc32_validation`] | persistence | A single byte flipped anywhere in the payload is caught: the read fails on the recorded checksum, or failing that on the structure the bytes no longer describe. Silent acceptance would be the worst outcome available here, because a subtly wrong precision matrix is harder to notice and harder to recover from than no checkpoint at all. |
//! | [`checkpoint_version_mismatch`] | persistence | A checkpoint whose header declares a format version this binary does not speak is refused outright, and refused on the version rather than on whatever the payload bytes happen to look like. The format carries no migration path by design, so the version field is the guard that stops an older or newer layout being reinterpreted as the current one. |
//! | [`checkpoint_invalid_magic`] | persistence | The first four bytes are checked before anything else is believed: a file that does not open with the checkpoint's magic is rejected as not being one, even when the rest of it is a perfectly valid checkpoint. Pointing the restore path at the wrong file should be a clean refusal, not an attempt to parse an unrelated file as model state. |
//! | [`checkpoint_atomic_write_cleanup`] | persistence | A completed write leaves the checkpoint in place and nothing else beside it: the temporary file the atomic protocol writes through is gone, having become the checkpoint by rename. The scratch file existing afterwards would mean the rename had not happened, which is exactly the state a crash is meant to be indistinguishable from. |
//! | [`checkpoint_mu_bit_exact`] | persistence | The mean vectors of all three core models — operational, sister and anchor — return bit-for-bit identical, not merely close. The restored model is meant to continue the old one rather than approximate it, and rounding introduced at the persistence boundary would accumulate silently over every restart the process ever performs. |
//! | [`checkpoint_precision_bit_exact`] | persistence | The operational model's covariance data survives the round trip bit-for-bit as well, packed triangle and all. Covariance is what the restored model reasons about uncertainty with, and it is reconstructed from these stored entries rather than recomputed, so any drift in them is drift in every confidence the model reports afterwards. |
//! | [`checkpoint_l024_fields_preserved`] | persistence | Each core model's numerical-health bookkeeping crosses the checkpoint intact: how many labels have passed since the last factorisation, the interval currently in force, the run of clean recomputes, and the last conditioning and synchronisation readings. These decide when the next recomputation is due, so losing them at a restart would reset the schedule and let a poorly conditioned model run longer than it should. |
//! | [`checkpoint_scalars_preserved`] | persistence | Deliberately distinct values assigned to the conditioning estimates, the two positive-class priors and the label sequence all come back as those exact values rather than as defaults. Testing with the values a cold start would produce anyway proves nothing; giving each field its own value is what shows the state is carried rather than reconstructed. |
//! | [`checkpoint_concordance_state_preserved`] | persistence | The concordance tracker crosses the checkpoint whole: the bounded window still holding its capacity of observations, the thresholds last calibrated, the total count of observations seen, and how many calibrations that count has triggered. Because the count outlives the window it was accumulated from, a restarted process resumes the recalibration rhythm mid-cycle instead of starting the interval over. |
//! | [`checkpoint_overwrite_existing`] | persistence | Checkpointing to a path that already holds one replaces it: a subsequent read sees the later state and no trace of the earlier. Checkpoints are taken repeatedly at the scheduler's interval onto the same path, so the steady-state behaviour of the writer is replacement, and a reader must never be able to find a stale snapshot lingering underneath. |
//! | [`checkpoint_empty_outcome_models`] | persistence | An instance where no outcome axis has been registered checkpoints and restores as having none — emptiness is recorded, not treated as missing data. A host that checkpoints before registering its axes must come back with an empty axis set rather than with a corrupt or absent one. |
//! | [`journal_50_entries`] | persistence | A long run of appended records reads back complete and in the order it was written, each entry still paired with the assessment it was about. The length-prefixed framing has to survive being applied fifty times in succession, because replay walks the file from the front and any record whose frame is misread takes every record after it with it. |
//! | [`journal_truncated_last_entry`] | persistence | A journal whose final record was cut short — the shape a crash mid-write leaves behind — gives up exactly that record and returns every complete one before it. The last label the process was accepting when it died is the one label it is allowed to lose; losing the other forty-nine along with it would make the journal worthless precisely when it is needed. |
//! | [`journal_empty_file`] | persistence | A journal file that exists but holds nothing reads as no entries at all rather than as an error. Having nothing to replay is the ordinary case after a clean checkpoint, and it must not be reported as a fault that pushes the caller into a cold start. |
//! | [`journal_nonexistent_file`] | persistence | cites (´claim:persistence:having-nothing-to-replay-reads-as-no-entries-not-as-a-failure´) |
//! | [`journal_truncate_resets_to_zero`] | persistence | Truncating at a mark the whole journal sits under empties it in both senses: the file drops to its bare format header and a subsequent read yields nothing. This is what a successful checkpoint does when every journalled entry has been absorbed — anything at or below the mark left behind would be replayed a second time on top of a model that already contains it. |
//! | [`journal_version_mismatch_discards_by_design`] | persistence | A journal of another format version is refused on the version and then discarded by design, exactly as a checkpoint is: the reader reports the mismatch rather than misreading records of another layout, and a writer opened over the mismatched file starts a fresh journal numbered from the checkpoint's mark. Construction survives the encounter — an upgraded binary meeting the previous deployment's journal cold-starts, it does not refuse to build (´dec:durability:checkpoint-journal´). |
//! | [`journal_truncate_to_retains_unabsorbed_tail`] | persistence | Truncation measures against the checkpoint's mark, not against the file: of five journalled entries with three absorbed, the two beyond the mark survive the truncation intact and the writer keeps numbering above them. A label appended after the checkpoint's capture is in the journal and not in the checkpoint, so a truncation that discarded the whole file would leave it in neither artefact — the silent loss (´dec:durability:checkpoint-journal´) exists to make impossible. |
//! | [`journal_append_one_entry`] | persistence | cites (´claim:persistence:a-run-of-appended-records-reads-back-complete-and-in-order´) |
//! | [`journal_entry_fields_round_trip`] | persistence | A label crosses the journal with every field as submitted: the assessment it names, the action the host actually took, a negative valence carried with its sign, and a ground-truth flag that stays false because it was never raised. Replay re-applies these values as the update, so a field flipped or defaulted in transit would teach the restored model something the host never said. |
//! | [`journal_entry_with_outcomes`] | persistence | Per-axis outcomes survive the journal still attached to the axes they were reported against, including axis identifiers far outside any contiguous range and values of either sign. The axis keys are what route each value to its own model on replay, so a mapping scrambled in transit would credit one axis with another's evidence. |
//! | [`journal_duplicate_assessment_id_allowed`] | persistence | The journal records what was submitted, not what was unique: a label journaled twice for the same assessment — the shape a host retry leaves after a journalled-but-not-enqueued result — appears twice, under distinct sequence numbers, with the unrelated label between them untouched. Writing is on the hot path and cannot afford to scan history for duplicates; deciding which attempt counts is deferred to replay, where the whole file is in hand anyway. |
//! | [`journal_large_context`] | persistence | A record carrying a thousand signal features round-trips with its full length and with an interior value still where it was put. The length prefix is a fixed-width field ahead of a variable payload, so the interesting question is whether a record much larger than the frame that describes it is still bounded correctly — a wide feature vector is an ordinary configuration, not an edge case. |
//! | [`journal_sentinel_extractions_round_trip`] | persistence | Each Sentinel's extraction survives the journal as its own triple — coordinate, features and occupancy — keyed by the Sentinel it came from, with an unoccupied extraction staying unoccupied. Replay routes each feature set back into its own Sentinel's graph at that coordinate, and the occupancy flag is what distinguishes a Sentinel that had nothing to say from one that said zero. |
//! | [`journal_writer_assigns_monotonic_seq`] | persistence | The writer, not the caller, decides sequence numbers: entries handed to it carrying nothing meaningful come out numbered one, two, three, and the file on disk agrees with the numbers it returned. Sequence is the ordering replay depends on and the coordinate the checkpoint's high-water mark is expressed in, so it can only be assigned at the single point where writes are serialised. |
//! | [`journal_writer_resumes_after_existing_entries`] | persistence | A writer opened over a journal that already holds entries reads the file before writing to it and resumes above the highest sequence it finds. A restarted process appends to the same journal the previous one left, and numbering from scratch would produce two records claiming the same position in a single replay ordering. |
//! | [`journal_writer_open_after_uses_max_of_checkpoint_and_journal`] | persistence | Resuming after a restore takes the higher of the two marks it has: the checkpoint's last processed sequence wins over a journal that only reaches a lower one. The two can disagree in either direction — a journal truncated after checkpointing lags, an unabsorbed tail leads — and only the maximum guarantees the next number is above everything the model has already seen. |
//! | [`journal_writer_open_empty_starts_at_one`] | persistence | Numbering begins at one on a journal that has never been written, leaving zero free to mean unsequenced. An entry still carrying zero has not been through the writer, and that distinction is only available if no real record ever occupies the value. |
//! | [`replay_dedup_filters_duplicate_assessment_id`] | persistence | Replay applies each assessment once however often it was journaled: five records naming four distinct assessments produce four updates and one deliberate skip. Deduplication is keyed on the assessment rather than on the sequence number, because the retry that produced the second record was a second attempt at the same event, not a second event. |
//! | [`replay_dedup_preserves_first_occurrence`] | persistence | Of two records for one assessment it is the earlier that is kept: replay walks in sequence order and the survivor is the one with the lower number. Keeping the later record instead would be a different and worse rule — the first attempt is the one the model may already have partly seen, and preferring the retry would make replay's result depend on how many times the host happened to retry. |
//! | [`replay_dedup_empty_input`] | persistence | Nothing to replay produces no updates: the dedup pass over an empty set of entries leaves the restored model exactly as the checkpoint described it. This is the common path after a clean shutdown, and it must not touch the model on the way past. |
//! | [`restore_validation_fields_belong_to_checkpoint_schema`] | persistence | Every checkpoint field the restore's structural validation reads is a field the checkpoint schema declares, with neither side of the comparison written down: the schema comes from the payload's own serialisation and the validated fields from the validation function's own body, bounded at its closing brace. The correspondence is then put under drift in both directions it can take — a schema that has lost a validated field is refused, once for each field the discovery found and each refusal naming the field it lost, and a validation that has gained a field the schema never declared is refused likewise. Two surfaces written in different places are what (´cav:durability:restore-field-mismatch´) says nothing makes agree; a test holding its own copy of either would keep passing through exactly the edit it exists to catch, so it holds neither. |
//! | [`recovery_with_time_decay`] | persistence | Restoring a day-old checkpoint succeeds and hands back a working copy at the dimensions it was saved with — the operational model at its full width, the anchor at its own narrower one. Ageing the state for the downtime happens on the way through, and it rescales what the models hold rather than reshaping them. |
//! | [`recovery_with_journal_replay`] | persistence | Recovery hands back only the journal entries the checkpoint had not already absorbed: against a snapshot taken at the fifth label, a journal holding ten yields the last five, starting exactly one past the mark. The checkpoint already contains the effect of everything up to its own sequence, so replaying those records would apply the same evidence a second time. |
//! | [`recovery_structural_mismatch`] | persistence | A checkpoint whose anchor model has a different width than this build uses is refused, even though the file itself is perfectly intact and passes every integrity check. Feature layout is fixed at construction, so state saved under a different geometry describes coefficients for positions that no longer mean what they meant; starting cold is the only honest option. |
//! | [`recovery_missing_checkpoint`] | persistence | Pointed at a checkpoint that does not exist, recovery declines rather than failing: the caller is told there is nothing to restore from and starts cold. Every first start on a fresh deployment takes this path, so an absent checkpoint has to be an ordinary answer rather than an error condition. |
//! | [`recovery_corrupt_checkpoint`] | persistence | cites (´claim:persistence:an-unusable-checkpoint-yields-a-cold-start-rather-than-a-refusal-to-run´) |
//! | [`recovery_version_mismatch_returns_none`] | persistence | cites (´claim:persistence:an-unusable-checkpoint-yields-a-cold-start-rather-than-a-refusal-to-run´) |
//! | [`recovery_preserves_last_processed_seq`] | persistence | The label high-water mark survives the whole restore path unchanged, not merely the file format: the working copy handed back reports the same sequence the checkpoint was taken at. That number is what the replay filter and the resumed writer both measure against, so it is the one field whose loss would silently double-apply or skip labels. |
//! | [`recovery_decay_applied_to_precision`] | persistence | Downtime costs the model confidence in the right direction: after restoring a day-old checkpoint the precision diagonal is lower than it was and the covariance diagonal higher. Evidence gathered before an outage describes a world that has since moved on, so the restored model must hold its beliefs more loosely — not merely hold them. |
//! | [`scheduler_sends_checkpoint_command`] | persistence | The scheduler's whole job is to ask: on each interval it puts a checkpoint request on the command channel and nothing more. Writing the file is the model owner's work, because only that thread holds the state consistently; a scheduler that wrote checkpoints itself would have to take a lock across the entire snapshot. |
//! | [`scheduler_shutdown_on_drop`] | persistence | Dropping the control channel ends the scheduler promptly even when its interval is an hour away: the thread is waiting on that channel with a timeout, not sleeping through it, so disconnection is noticed at once. A scheduler that only checked for shutdown when its timer fired would hold up every shutdown by up to a full interval. |
//! | [`scheduler_thread_name`] | persistence | The scheduler thread carries its instance's name alongside its role. Where several instances share a process, a thread dump or profile that named them all identically would leave an operator unable to tell whose checkpointing is stalling. |
//! | [`assayer_with_persistence_constructs`] | persistence | Turning persistence on changes nothing about how an instance comes up: it builds against an empty directory and publishes its first snapshot exactly as a non-persistent one does. The extra thread and the extra files are not a precondition for serving. |
//! | [`assayer_drop_shuts_down_scheduler_first`] | persistence | Dropping a persistent instance returns rather than hanging. Shutdown has to unwind two threads that hold channels to each other, and the ordering matters: were the model owner to stop first, a scheduler still asking it for checkpoints would keep the drop waiting forever. |
//! | [`journal_open_failure_refuses_the_build`] | persistence | A journal path that cannot be opened fails the construction rather than aborting the process: the caller gets `PersistenceSetupFailed`, the variant whose documentation describes this exact condition. Construction is the package's fallible surface and an unwritable path is a caller-supplied configuration the contract does not admit, so it belongs on the error channel that already names it. The ordering matters as much as the channel — the open is attempted before any thread is spawned, so a refused build leaves nothing running behind it. |
//! | [`construction_numbers_above_the_checkpoint_mark`] | persistence | After a restart over a checkpoint-truncated journal, the writer numbers above the checkpoint's mark rather than from one. The mark is the highest sequence the model has already absorbed, and the replay filter keeps only entries above it, so a writer that restarted below it would journal a whole run's labels into the range replay discards — acknowledging them as durable and then reading, parsing and passing over them on the next start. |
//! | [`checkpoint_with_ledger`] | persistence | Each Sentinel's outcome ledger crosses the checkpoint entire — every entry including the root, under its own Sentinel, with the stored rates intact to the last bit. The ledger is a long-memory structure built from many assessments; rebuilding it from scratch after a restart would take as long as it took to learn. |
//! | [`checkpoint_with_identity`] | persistence | A dimension's identity state crosses the checkpoint under its own dimension with its competitive cells and one outcome record per cell. The cells and their outcomes are stored as parallel structures and must arrive as a matched set: a restored dimension holding cells it has no outcome state for would be a dimension that cannot say anything about the entities it routes. |
//! | [`checkpoint_with_cell_outcome_state`] | persistence | Cell outcome rates survive the checkpoint value for value and cell by cell: five cells given five distinct rates come back with each rate still on the cell it belonged to, matching to the last representable digit. Counting the cells is not enough — what makes a cell useful is the rate attached to it, and a permutation would be invisible to any count. |
//! | [`checkpoint_format_version_bump`] | persistence | cites (´claim:persistence:a-checkpoint-of-another-format-version-is-refused-on-the-version´) |
//! | [`checkpoint_prior_version_refused`] | persistence | cites (´claim:persistence:a-checkpoint-of-another-format-version-is-refused-on-the-version´) |
//! | [`checkpoint_ledger_decay_on_restore`] | persistence | Stored averages are aged once, in bulk, at the moment of restore: a ledger rate saved a day ago comes back smaller than it was but still positive, and its own last-updated stamp is moved forward to now. Both halves matter together — the decay accounts for the downtime, and restamping the clock is what stops the same gap being charged again the next time that entry is touched. |
//! | [`restore_measures_downtime_on_the_injected_clock`] | persistence | Downtime is measured on the clock the engine was given, not on the wall. A checkpoint captured and restored under an injected clock that advanced exactly a day is aged by exactly a day's worth of decay, to the last place the arithmetic holds. The two ends of that subtraction have to come from one time domain. The checkpoint's timestamp is written from the engine's clock, so a restore that read the wall clock was subtracting a virtual instant from a real one: the gap it computed was the distance between the two domains rather than the downtime, and at the scale that distance reaches it is the elapsed ceiling that answers, ageing every stored average by a year of absence that never happened. A deterministic replay would then depend on the day it was run. |
//! | [`restore_refuses_a_journal_corrupt_before_its_tail`] | persistence | A journal corrupt before its tail fails the restore instead of yielding an empty replay set, while the two failures the protocol absorbs still restore cleanly: a record torn at the tail costs only itself, and a journal of another format version is discarded by design. The entries past the checkpoint's mark are the only copy of every label acknowledged since the capture. A corruption before the tail makes every record after it unfindable, so proceeding with no replay would discard evidence the host was told was durable and report a successful start over the gap. The host has real actions at that moment — an earlier checkpoint, a repaired journal, a deliberate cold start — and construction has to fail for it to take one knowingly. The torn tail is the opposite case and stays absorbed: it is the single label the append protocol allows to be lost, and refusing the build over it would turn every crash into an outage. So is the version mismatch, which is how an upgraded binary meets the previous deployment's journal. |
//! | [`checkpoint_identity_decay_on_restore`] | persistence | cites (´claim:persistence:stored-averages-are-aged-once-on-restore-and-their-clocks-restamped´) |
//! | [`checkpoint_complete_round_trip`] | persistence | The parts of the state travel together rather than merely each travelling: a checkpoint carrying core scalars, a registered outcome axis with its spatial flag, two Sentinel ledgers of different sizes, a dimension's identity state and the dimension map restores all of them at once, and the working copy rebuilt from it agrees on the total feature width. It is the agreement between the pieces that makes the state usable — a dimension map that no longer matches the models it indexes would restore cleanly and then be wrong. |
//! | [`checkpoint_axis_fields_round_trip`] | persistence | An axis registered away from every default comes back from a restart as it was registered: the name, the description and the all-labels eligibility mode all cross the checkpoint and are read back off the restored working copy, not substituted. The eligibility mode decides which labels the axis model trains on (´def:axis:training-target´), so an axis silently reverted to the eligible-only default would train on a differently-selected population from the restart onward with no surface reporting it. |
//! | [`checkpoint_drift_state_round_trip`] | persistence | Drift evidence survives a restart: accumulators fed to distinct values on three models cross the checkpoint and come back on the rebuilt working copy value for value — both CUSUM sides, both smoothed diagnostics and the step counter, each still on the model that accumulated it. A restart is not among the four discard triggers (´tab:monitoring:drift-resets´), so a restore that handed every drifting model a fresh start would be a reset the table does not admit, firing on every deployment's ordinary cadence. |
//! | [`checkpoint_standardisation_round_trip`] | persistence | The standardisation statistics cross the checkpoint as values, not as shapes: means, variances and the class vector all come back on the rebuilt working copy exactly as they stood at capture, with the class assignments still on the positions they described. Every feature every model reads is scaled by these vectors, so a restore that reset them to the neutral pair would mis-scale the whole feature vector for the standardisation half-life after every restart (´req:standardisation:timing´). |
//! | [`checkpoint_mid_ramp_resumes_where_it_stopped`] | persistence | A checkpoint taken part way along the cold ramp comes back as the same ramp: the base it was mixing against, the sample it had accepted and the count that positions it all cross the file, and the restored instance publishes exactly the moments the running one published and reports exactly the same phase and count. A checkpoint that carried the published pair and nothing behind it would leave a restore only two answers, both wrong — restart the ramp, and pay the whole cold start again at every restart; or treat a partial transition as a finished one, and publish a mixture as though it were the empirical moments. Carrying it whole is what makes a resumed ramp publish what an uninterrupted one would have published at the same count, rather than one run's sample against another run's assumptions. |
//! | [`checkpoint_ramp_recomputes_against_the_configured_horizon`] | persistence | A ramp that had reached its horizon comes back in service, and a ramp restored against a horizon that has since been shortened completes on arrival and publishes the empirical moments. The horizon is deliberately not carried in the file: it is read from the configuration in force, so a restore recomputes its own position rather than holding a second authority that could disagree with the configuration it was restored under. A count at or past the new horizon has no prior mass left to retire whatever it was gathered against, and the one-time change in mixture weight is disclosed by the restore's own coordinate-version change. |
//! | [`checkpoint_standardisation_warm_start_end_to_end`] | persistence | A restarted instance publishes the standardisation it had learned, through the path a deployment takes: labels move the running statistics, an acknowledged checkpoint captures them, and the instance rebuilt from that file publishes the same vectors in its first snapshot — with the batch initialisation left off, its premise now true rather than asserted. This is the warm-start branch's ground made real: the fast path is switched off because the restored statistics are empirical, not because a comment says they are (´alg:standardisation:batch-initialisation´). |
//! | [`checkpoint_structural_mismatch_sentinel_count`] | persistence | A checkpoint carrying more Sentinels than the restoring instance knows about restores anyway, and all of that ledger state is handed back for the caller to reconcile. Sentinels are registered at runtime rather than fixed at construction, so their population is not part of what makes a checkpoint structurally compatible — only the feature geometry is. Discarding a Sentinel's history because it had not re-registered yet would throw away state that is about to become relevant again. |
//! | [`checkpoint_structural_mismatch_dimension_count`] | persistence | cites (´claim:persistence:runtime-registered-populations-are-not-part-of-the-structural-check´) |
//! | [`checkpoint_round_trip_via_assayer`] | persistence | A checkpoint taken through a running instance records the work it had actually done: after ten labels have been processed and an explicit checkpoint acknowledged, the file on disk names ten as its high-water mark and carries the models at their configured widths. The acknowledgement is what makes this meaningful — it is the model owner's statement that the snapshot includes everything it had accepted, and it is the only thing distinguishing a checkpoint from a request for one. |
//! | [`checkpoint_whole_state_production_path`] | persistence | The checkpoint a deployment actually writes is whole-state: with no payload constructed by hand, a running instance's acknowledged checkpoint captures the Sentinel ledger it holds (a distinctive rate crossing value for value), the identity dimension's published graph at the same energy the live snapshot reports, the calibration buffer row for row, the Platt tracker, the label counter and the health counters — and the rebuilt instance serves that state: the restored ledger answers with the same rate, and the first label after the restart counts from where the run left off rather than from zero. This is the path the six hand-payload tests establish capability for, exercised end to end (´dec:durability:checkpoint-journal´). |
//! | [`journal_replay_via_assayer_recovery`] | persistence | cites (´claim:persistence:only-entries-beyond-the-checkpoints-mark-are-handed-back-for-replay´) |
//! | [`checkpoint_round_trip_end_to_end`] | persistence | An instance rebuilt from its own checkpoint comes up serving and keeps learning from where it left off: it registers, assesses and returns a finite risk on the first request, and ten further labels are all processed and counted from the fifty the checkpoint captured rather than from zero. Nothing about the restored state has to be re-taught before the model is usable again — the point of persistence is that the restart is invisible to the host on the other side of it. |
//! | [`checkpoint_restores_concordance_tracker`] | persistence | The health a rebuilt instance reports is the health it had before shutdown: window occupancy, total observations, calibrations completed and the current thresholds all read the same on either side of the restart. Concordance is how an operator judges whether the model is behaving, and a report that silently reset at every deployment would make the metric mean nothing across exactly the events most worth watching. |
//! | [`shutdown_after_checkpoint_trigger`] | persistence | cites (´claim:persistence:dropping-a-persistent-instance-unwinds-both-threads-without-hanging´) |
//! | [`label_channel_full_with_journal`] | persistence | When the model owner cannot keep up, a journalling instance tells the host precisely which of two things happened: the label was written down but not handed on. It is never reported as a plain full channel, because the journal mutex spans both the append and the hand-off, so any label that reached the hand-off is already durable. The distinction is what the host retries on — a durable label may be re-submitted safely, since replay will keep only the first attempt. |
//! | [`concurrent_label_preserves_journal_channel_order`] | persistence | Labels submitted from eight threads at once land in the journal in strictly increasing sequence order, every one of them present, and the mark the model owner subsequently checkpoints at is exactly the last sequence on disk. The two facts together are what make replay safe: the ordering on disk matches the order the owner received them, so the high-water mark cannot advance past an entry that has not been applied. Were the mutex released between writing a record and handing it on, a thread holding an earlier sequence could be overtaken, the owner would raise its mark past that entry, and a later crash would skip the overtaken label on replay without any trace of having done so. |
//! | [`a_checkpoint_without_a_baseline_restores_without_one`] | persistence | A checkpoint written before the rebuild's baseline existed carries none, and the restore says so rather than inventing one: the model comes back with the baseline absent, both shares of the precision matrix the prior holds up at nothing, the coordinate-shaped share widened to the model's own width, and the per-label check falling back to its counter because it has nothing to be relative to. The first rebuild after the restore establishes the record, at which point every relative arm becomes available again. A restore that fabricated a baseline would put the check to work against a state no rebuild had measured. The two components of the synchronisation reading come back absent for the same reason and by two different routes: the record that would carry them is not there, and the share of the clamp mass the covariance has not been rebuilt against is not persisted at all, because it is a fact about a pair rather than about a model and the pair the checkpoint restores is one the rebuild already agreed with. A restored model therefore attributes none of its drift to the prior until it has rebuilt once, which errs toward the cadence reacting to drift it cannot account for rather than away from it. |
//! | [`checkpoint_restore_refuses_the_indefinite_witness`] | persistence | The restore reads the spectrum where the storage boundary read only the diagonal, and the witness is the suite's own: ones on the diagonal and twos off it, whose minimum diagonal is strictly positive and whose least eigenvalue is minus one. Both halves stand here rather than only the second, because a witness that failed the diagonal test would witness nothing — the whole content of the repair is that the cheap test says yes exactly where the restore now says no. The refusal names the pivot the factorisation stopped at, so a reader inspecting the artefact is pointed at a coordinate rather than at a file. |
//! | [`checkpoint_restore_admits_a_healthy_precision_matrix`] | persistence | A checkpoint the model may hold restores through the same call untouched: the verdict is a gate and not a transformation. The mean, the precision and the covariance come back bit for bit, and the cadence fields come back as the values that were stored rather than as the defaults a fresh model would carry. Asserting the restore still works is not redundant beside the refusal — a verdict that refused everything would satisfy the refusal test alone. |
//! | [`recovery_refuses_an_indefinite_checkpoint`] | persistence | A checkpoint that is read, structurally agreed and then refused on its numbers fails the restore rather than becoming a cold start, and the error is the builder's own. The distinction is the whole of the repair: an unreadable or structurally incompatible checkpoint is a deployment with nothing to resume and cold-starting costs it nothing it had, while a checkpoint whose precision matrix the model may not hold is corruption, and cold-starting on it would discard every learned parameter at the one moment the host could still reach for an earlier checkpoint or the journal. |
//! | [`the_repaired_count_crosses_the_checkpoint`] | persistence | The repaired count crosses the checkpoint, and it crosses it in the slot the retired repair cascade's own count occupied. The payload's encoding is positional rather than named, so the rename moved no byte and the layout generation did not turn on it (the suite's version-mismatch test pins the generation at what it was); what a reader gets back is the count the model had. The residual this leaves is stated at the record rather than hidden here: a checkpoint written before the counter's subject moved restores its old reading into the new field, which is admitted because both readings answer the one question the counter has always asked (´dec:durability:structural-compatibility´). |
//! | [`a_model_state_that_disagrees_with_its_own_width_is_refused`] | persistence | A checkpoint whose model state declares one width and carries a mean, a precision matrix, a covariance or a clamp mass of another is refused as an error, and the refusal names the model and both disagreeing extents. The checksum the reader takes is an integrity check and not a consistency one, and the structural check taken after it compares the artefact against this build rather than against itself, so an internally inconsistent state passes both; the conversion that widens the flat covariance back into a matrix on the declared width then asserts in every build, and a defective artefact ends the process at startup instead of leaving the host the three actions the refusal exists to leave it (´dec:degradation:error-partition´). |
//! | [`an_outcome_axis_state_that_disagrees_with_its_own_width_is_refused`] | persistence | The width check reaches every registered outcome axis and not only the three fixed models: an axis state one covariance entry short of its own declared width is refused, and the refusal names that axis. The axes are the part of the payload whose membership the host decides at runtime, so they are both the part a checkpoint carries an unbounded number of and the part a check written against the fixed three would silently not cover. |

#![allow(clippy::float_cmp, clippy::match_same_arms, clippy::cast_precision_loss)]
//! Crate-level tests for the persistence module (´dec:durability:checkpoint-journal´).
//!
//! Persistence is what stands between a restart and a cold model. The
//! checkpoint is a full, self-validating snapshot written atomically; the
//! journal is the append-only record of labels accepted since that snapshot;
//! recovery is the protocol that puts the two back together, ages the restored
//! state by the downtime, and hands back only the journal entries the
//! checkpoint had not yet absorbed. Every failure in that path is designed to
//! degrade to a cold start rather than to a refusal to run.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::time::Duration;

use crate::config::types::AssayerConfig;
use crate::owner::commands::{LabelData, PendingContext};
use crate::pending::SentinelExtraction;
use crate::persistence::checkpoint::{self, CheckpointPayload};
use crate::persistence::journal::{self, JournalEntry};
use crate::persistence::{recovery, scheduler};
use crate::snapshot::working::WorkingCopy;
use crate::testing::registrations::sentinel_reg;
use crate::testing::{ACK_DEADLINE, LabelSpec};
use crate::types::{Action, AssessmentId, ChannelId, EntityKey, OutcomeAxisId, PersistentTimestamp, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Creates a temp directory for tests, returns the path.
fn test_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("failed to create temp dir")
}

/// Creates a cold-start working copy with small dimensions.
fn test_working_copy() -> WorkingCopy {
    WorkingCopy::cold_start(16, &super::helpers::test_model_config_with_anchor(5), 1000)
}

/// Creates a test AssayerConfig suitable for persistence tests.
///
/// If `persistence_dir` is provided, persistence will be enabled with checkpoint
/// and journal files in that directory.
fn test_persistence_config(persistence_dir: Option<&std::path::Path>) -> AssayerConfig {
    use crate::config::types::{InfrastructureConfig, PersistenceConfig};

    let persistence = persistence_dir.map(|dir| PersistenceConfig {
        checkpoint_dir: dir.to_path_buf(),
        journal_dir: dir.to_path_buf(),
        checkpoint_interval: std::time::Duration::from_secs(3600),
    });

    AssayerConfig {
        instance_id: "test".to_owned(),
        model: super::helpers::test_model_config_with_anchor(5),
        infrastructure: InfrastructureConfig {
            label_channel_capacity: 100,
            ..crate::testing::test_infrastructure()
        },
        persistence,
        ..Default::default()
    }
}

/// Creates a checkpoint payload from a working copy.
fn test_payload(working: &WorkingCopy) -> CheckpointPayload {
    working.to_checkpoint_payload(PersistentTimestamp::now(), Vec::new(), Vec::new())
}

/// The field names the checkpoint schema declares, read off the payload's own
/// serialisation rather than off a transcription of the structure. A field
/// added to or removed from `CheckpointPayload` changes this set on the next
/// build with nothing here to update.
fn checkpoint_schema_fields(payload: &CheckpointPayload) -> BTreeSet<String> {
    serde_json::to_value(payload)
        .expect("checkpoint payload serialises")
        .as_object()
        .expect("checkpoint payload serialises as an object")
        .keys()
        .cloned()
        .collect()
}

/// The body of the function whose signature opens `text`, delimited by the
/// braces that open and close it.
///
/// Bounding the window at the closing brace is what keeps the discovery about
/// the validation function alone. A scrape that ran to the end of the file
/// would quietly take in the payload reads of whatever function was appended
/// after it, and would report fields the restore validation never touches.
fn brace_delimited_body(text: &str) -> &str {
    let open = text.find('{').expect("the validation function has a body");
    let mut depth = 0_usize;
    for (offset, byte) in text.bytes().enumerate().skip(open) {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &text[open + 1..offset];
                }
            }
            _ => {}
        }
    }
    panic!("the validation function's body is closed");
}

/// The checkpoint fields the live restore validation reads, discovered from
/// that function's own source text rather than from a list kept beside it.
///
/// The source is pulled in with `include_str!`, so the validation function is
/// a build input of this test: editing it rebuilds and re-discovers.
fn restore_validation_fields() -> BTreeSet<String> {
    let source = include_str!("../persistence/recovery.rs");
    let signature = "fn validate_structural_compatibility(";
    let mut definitions = source.match_indices(signature);
    let (start, _) = definitions.next().expect("restore validation function exists");
    assert!(definitions.next().is_none(), "restore validation function is unique");

    brace_delimited_body(&source[start..])
        .split("payload.")
        .skip(1)
        .map(|suffix| {
            let field_len = suffix
                .as_bytes()
                .iter()
                .take_while(|byte| byte.is_ascii_alphanumeric() || **byte == b'_')
                .count();
            assert!(field_len > 0, "a payload access names its field");
            suffix[..field_len].to_owned()
        })
        .collect()
}

/// The correspondence oracle: every validated field must be one the schema
/// declares, and a refusal names the fields that are not.
fn require_declared_restore_fields(
    checkpoint_fields: &BTreeSet<String>,
    validated_fields: &BTreeSet<String>,
) -> Result<(), String> {
    let missing: Vec<_> = validated_fields.difference(checkpoint_fields).cloned().collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "restore validates undeclared checkpoint fields: {}",
            missing.join(", ")
        ))
    }
}

/// Creates a test label data.
fn test_label(assessment_id: AssessmentId) -> LabelData {
    LabelSpec::new(assessment_id).valence(1.0).ground_truth().build()
}

fn test_pending_context(id: AssessmentId) -> PendingContext {
    PendingContext {
        spatial_axis_ids: Vec::new(),
        entity: EntityKey::new(id.0.to_le_bytes().to_vec()),
        sentinel_extractions: HashMap::new(),
        identity_coordinates: HashMap::new(),
        identity_active_cells: HashMap::new(),
        active_sentinels: Vec::new(),
        reporting_sentinels: Vec::new(),
        entity_base_features: HashMap::new(),
        entity_axis_features: HashMap::new(),
        signal_features: crate::pending::StoredFeatures::default(),
        risk_basis: crate::pending::PendingRiskBasis::default(),
        outcome_predictions: HashMap::new(),
    }
}

/// Creates a test journal entry.
fn test_journal_entry(seq: u64) -> JournalEntry {
    JournalEntry {
        seq,
        label: test_label(AssessmentId(seq)),
        context: test_pending_context(AssessmentId(seq)),
        timestamp: PersistentTimestamp::now(),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Checkpoint Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A checkpoint written to disk and read back yields the state that was saved:
/// the label sequence, the four calibration scalars and the shape and mean of
/// each core model all come back as they went in. This is the whole premise of
/// a warm restart — the file has to be a faithful account of the model, not an
/// approximation of it.
///
/// ´claim:persistence:a-checkpoint-read-back-yields-the-state-that-was-written´
/// ´test:crate:checkpoint-round-trip´
#[test]
fn checkpoint_round_trip() {
    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let working = test_working_copy();
    let payload = test_payload(&working);

    checkpoint::write_checkpoint(&path, &payload).expect("write failed");
    let restored = checkpoint::read_checkpoint(&path).expect("read failed");

    // Compare key fields.
    assert_eq!(restored.last_processed_label_seq, payload.last_processed_label_seq);
    assert_eq!(restored.kappa_v, payload.kappa_v);
    assert_eq!(restored.kappa_sister, payload.kappa_sister);
    assert_eq!(restored.kappa_anchor, payload.kappa_anchor);
    assert_eq!(restored.p_positive_global, payload.p_positive_global);
    assert_eq!(restored.p_positive_eligible, payload.p_positive_eligible);

    // Compare model parameters.
    assert_eq!(restored.operational.parameters.p, payload.operational.parameters.p);
    assert_eq!(restored.operational.parameters.mu, payload.operational.parameters.mu);
    assert_eq!(restored.sister.parameters.p, payload.sister.parameters.p);
    assert_eq!(restored.anchor.parameters.p, payload.anchor.parameters.p);
}

/// The marginalisation ledger crosses the file and rebuild boundary whole:
/// every event class, the fallback and certificate counters, the accumulated
/// discarded mass and the worst loss fraction return with their distinctive
/// values. These counters describe the deployment rather than one process, so
/// a restart must not make approximation already committed disappear from the
/// health surface.
///
/// ´claim:persistence:the-marginalisation-error-ledger-survives-a-checkpoint-and-restore´
/// ´test:crate:checkpoint-restores-marginalisation-error-ledger´
#[test]
fn checkpoint_restores_marginalisation_error_ledger() {
    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let mut working = test_working_copy();
    working.marginalisation_errors.events = 17;
    working.marginalisation_errors.measured_events = 5;
    working.marginalisation_errors.bounded_events = 7;
    working.marginalisation_errors.unbounded_events = 5;
    working.marginalisation_errors.fallback_events = 3;
    working.marginalisation_errors.verified_events = 13;
    working.marginalisation_errors.gershgorin_certified_events = 11;
    working.marginalisation_errors.cumulative_discarded_precision_mass = 23.5;
    working.marginalisation_errors.worst_correction_loss_fraction = 0.625;
    let expected = working.marginalisation_errors.clone();

    let payload = test_payload(&working);
    checkpoint::write_checkpoint(&path, &payload).expect("write failed");
    let restored_payload = checkpoint::read_checkpoint(&path).expect("read failed");
    let restored = WorkingCopy::from_checkpoint_payload(&restored_payload, 100, 0.1)
        .expect("the fixture checkpoint carries definite precision matrices");

    assert_eq!(restored.marginalisation_errors, expected);
}

/// A single byte flipped anywhere in the payload is caught: the read fails on
/// the recorded checksum, or failing that on the structure the bytes no longer
/// describe. Silent acceptance would be the worst outcome available here,
/// because a subtly wrong precision matrix is harder to notice and harder to
/// recover from than no checkpoint at all.
///
/// ´claim:persistence:a-single-corrupted-payload-byte-fails-the-read-rather-than-restoring´
/// ´test:crate:checkpoint-crc32-validation´
#[test]
fn checkpoint_crc32_validation() {
    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let working = test_working_copy();
    let payload = test_payload(&working);
    checkpoint::write_checkpoint(&path, &payload).expect("write failed");

    // Corrupt one byte in the payload (after header).
    let mut data = fs::read(&path).unwrap();
    if data.len() > 12 {
        data[13] ^= 0xFF;
    }
    fs::write(&path, &data).unwrap();

    match checkpoint::read_checkpoint(&path) {
        Err(checkpoint::CheckpointError::IntegrityFailed { .. }) => {}
        Err(checkpoint::CheckpointError::Deserialise(_)) => {
            // Also acceptable: CRC might happen to match but payload is corrupt.
        }
        other => panic!("expected IntegrityFailed or Deserialise, got {other:?}"),
    }
}

/// A checkpoint whose header declares a format version this binary does not
/// speak is refused outright, and refused on the version rather than on
/// whatever the payload bytes happen to look like. The format carries no
/// migration path by design, so the version field is the guard that stops an
/// older or newer layout being reinterpreted as the current one.
///
/// ´claim:persistence:a-checkpoint-of-another-format-version-is-refused-on-the-version´
/// ´test:crate:checkpoint-version-mismatch´
#[test]
fn checkpoint_version_mismatch() {
    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let working = test_working_copy();
    let payload = test_payload(&working);
    checkpoint::write_checkpoint(&path, &payload).expect("write failed");

    // Tamper with the version field (bytes 4..8).
    let mut data = fs::read(&path).unwrap();
    data[4] = 0xFF;
    data[5] = 0xFF;
    data[6] = 0xFF;
    data[7] = 0xFF;
    fs::write(&path, &data).unwrap();

    match checkpoint::read_checkpoint(&path) {
        Err(checkpoint::CheckpointError::VersionMismatch { .. }) => {}
        other => panic!("expected VersionMismatch, got {other:?}"),
    }
}

/// The first four bytes are checked before anything else is believed: a file
/// that does not open with the checkpoint's magic is rejected as not being one,
/// even when the rest of it is a perfectly valid checkpoint. Pointing the
/// restore path at the wrong file should be a clean refusal, not an attempt to
/// parse an unrelated file as model state.
///
/// ´claim:persistence:a-file-lacking-the-magic-is-rejected-as-not-a-checkpoint´
/// ´test:crate:checkpoint-invalid-magic´
#[test]
fn checkpoint_invalid_magic() {
    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let working = test_working_copy();
    let payload = test_payload(&working);
    checkpoint::write_checkpoint(&path, &payload).expect("write failed");

    // Tamper with the magic bytes.
    let mut data = fs::read(&path).unwrap();
    data[0] = b'X';
    fs::write(&path, &data).unwrap();

    match checkpoint::read_checkpoint(&path) {
        Err(checkpoint::CheckpointError::InvalidMagic { .. }) => {}
        other => panic!("expected InvalidMagic, got {other:?}"),
    }
}

/// A completed write leaves the checkpoint in place and nothing else beside
/// it: the temporary file the atomic protocol writes through is gone, having
/// become the checkpoint by rename. The scratch file existing afterwards would
/// mean the rename had not happened, which is exactly the state a crash is
/// meant to be indistinguishable from.
///
/// ´claim:persistence:a-completed-write-leaves-the-checkpoint-and-no-scratch-file´
/// ´test:crate:checkpoint-atomic-write-cleanup´
#[test]
fn checkpoint_atomic_write_cleanup() {
    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");
    let tmp_path = dir.path().join("checkpoint.new");

    let working = test_working_copy();
    let payload = test_payload(&working);
    checkpoint::write_checkpoint(&path, &payload).expect("write failed");

    // The .new temp file should not exist after successful write.
    assert!(!tmp_path.exists(), ".new temp file was not cleaned up");
    assert!(path.exists(), "checkpoint file was not created");
}

/// The mean vectors of all three core models — operational, sister and anchor
/// — return bit-for-bit identical, not merely close. The restored model is
/// meant to continue the old one rather than approximate it, and rounding
/// introduced at the persistence boundary would accumulate silently over every
/// restart the process ever performs.
///
/// ´claim:persistence:model-means-return-bit-for-bit-identical-across-all-three-models´
/// ´test:crate:checkpoint-mu-bit-exact´
#[test]
fn checkpoint_mu_bit_exact() {
    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let working = test_working_copy();
    let payload = test_payload(&working);
    checkpoint::write_checkpoint(&path, &payload).expect("write failed");
    let restored = checkpoint::read_checkpoint(&path).expect("read failed");

    assert_eq!(
        restored.operational.parameters.mu, payload.operational.parameters.mu,
        "operational μ not bit-exact"
    );
    assert_eq!(
        restored.sister.parameters.mu, payload.sister.parameters.mu,
        "sister μ not bit-exact"
    );
    assert_eq!(
        restored.anchor.parameters.mu, payload.anchor.parameters.mu,
        "anchor μ not bit-exact"
    );
}

/// The operational model's covariance data survives the round trip bit-for-bit
/// as well, packed triangle and all. Covariance is what the restored model
/// reasons about uncertainty with, and it is reconstructed from these stored
/// entries rather than recomputed, so any drift in them is drift in every
/// confidence the model reports afterwards.
///
/// ´claim:persistence:the-operational-covariance-returns-bit-for-bit-identical´
/// ´test:crate:checkpoint-precision-bit-exact´
#[test]
fn checkpoint_precision_bit_exact() {
    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let working = test_working_copy();
    let payload = test_payload(&working);
    checkpoint::write_checkpoint(&path, &payload).expect("write failed");
    let restored = checkpoint::read_checkpoint(&path).expect("read failed");

    assert_eq!(
        restored.operational.parameters.covariance_data, payload.operational.parameters.covariance_data,
        "operational Σ not bit-exact"
    );
}

/// Each core model's numerical-health bookkeeping crosses the checkpoint
/// intact: how many labels have passed since the last factorisation, the
/// interval currently in force, the run of clean recomputes, and the last
/// conditioning and synchronisation readings. These decide when the next
/// recomputation is due, so losing them at a restart would reset the schedule
/// and let a poorly conditioned model run longer than it should.
///
/// ´claim:persistence:each-models-recomputation-bookkeeping-crosses-the-checkpoint-intact´
/// ´test:crate:checkpoint-l024-fields-preserved´
#[test]
fn checkpoint_l024_fields_preserved() {
    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let working = test_working_copy();
    let payload = test_payload(&working);
    checkpoint::write_checkpoint(&path, &payload).expect("write failed");
    let restored = checkpoint::read_checkpoint(&path).expect("read failed");

    // Verify the recomputation-cadence tracking fields round-trip for each
    // model (´alg:gaussian:condition-adaptive-recompute´).
    for (label, orig, rest) in [
        ("operational", &payload.operational, &restored.operational),
        ("sister", &payload.sister, &restored.sister),
        ("anchor", &payload.anchor, &restored.anchor),
    ] {
        assert_eq!(
            rest.labels_since_recompute, orig.labels_since_recompute,
            "{label} labels_since_recompute"
        );
        assert_eq!(
            rest.n_recompute_effective, orig.n_recompute_effective,
            "{label} n_recompute_effective"
        );
        assert_eq!(
            rest.consecutive_clean_recomputes, orig.consecutive_clean_recomputes,
            "{label} consecutive_clean_recomputes"
        );
        assert_eq!(
            rest.last_diagonal_ratio, orig.last_diagonal_ratio,
            "{label} last_diagonal_ratio"
        );
        assert_eq!(rest.last_sync_error, orig.last_sync_error, "{label} last_sync_error");
        assert_eq!(
            rest.sync_error_shortenings, orig.sync_error_shortenings,
            "{label} sync_error_shortenings"
        );
    }
}

/// Deliberately distinct values assigned to the conditioning estimates, the
/// two positive-class priors and the label sequence all come back as those
/// exact values rather than as defaults. Testing with the values a cold start
/// would produce anyway proves nothing; giving each field its own value is what
/// shows the state is carried rather than reconstructed.
///
/// ´claim:persistence:calibration-scalars-and-the-label-sequence-return-as-the-values-set´
/// ´test:crate:checkpoint-scalars-preserved´
#[test]
fn checkpoint_scalars_preserved() {
    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let mut working = test_working_copy();
    working.kappa_v = 2.5;
    working.kappa_sister = 3.0;
    working.kappa_anchor = 1.5;
    working.p_positive_global = 0.42;
    working.p_positive_eligible = 0.38;
    working.last_processed_label_seq = 99;

    let payload = working.to_checkpoint_payload(PersistentTimestamp::now(), Vec::new(), Vec::new());
    checkpoint::write_checkpoint(&path, &payload).expect("write failed");
    let restored = checkpoint::read_checkpoint(&path).expect("read failed");

    assert_eq!(restored.kappa_v, 2.5);
    assert_eq!(restored.kappa_sister, 3.0);
    assert_eq!(restored.kappa_anchor, 1.5);
    assert_eq!(restored.p_positive_global, 0.42);
    assert_eq!(restored.p_positive_eligible, 0.38);
    assert_eq!(restored.last_processed_label_seq, 99);
}

/// The concordance tracker crosses the checkpoint whole: the bounded window
/// still holding its capacity of observations, the thresholds last calibrated,
/// the total count of observations seen, and how many calibrations that count
/// has triggered. Because the count outlives the window it was accumulated
/// from, a restarted process resumes the recalibration rhythm mid-cycle
/// instead of starting the interval over.
///
/// ´claim:persistence:the-concordance-window-thresholds-and-counters-all-cross-the-checkpoint´
/// ´test:crate:checkpoint-concordance-state-preserved´
#[test]
fn checkpoint_concordance_state_preserved() {
    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let working = test_working_copy();
    let tracker = crate::health::ConcordanceTracker::new(crate::health::ConcordanceConfig {
        window_capacity: 4,
        recalibration_interval: 2,
        calibration_percentile: 80.0,
    });
    for i in 0..6 {
        tracker.observe([f64::from(i); 4]);
    }

    let mut payload = test_payload(&working);
    payload.concordance_state = tracker.checkpoint_state();

    checkpoint::write_checkpoint(&path, &payload).expect("write failed");
    let restored = checkpoint::read_checkpoint(&path).expect("read failed");

    assert_eq!(restored.concordance_state.window.len(), 4);
    assert_eq!(restored.concordance_state.thresholds, payload.concordance_state.thresholds);
    assert_eq!(restored.concordance_state.count, 6);
    assert_eq!(restored.concordance_state.calibrations_completed, 3);
}

/// Checkpointing to a path that already holds one replaces it: a subsequent
/// read sees the later state and no trace of the earlier. Checkpoints are
/// taken repeatedly at the scheduler's interval onto the same path, so the
/// steady-state behaviour of the writer is replacement, and a reader must
/// never be able to find a stale snapshot lingering underneath.
///
/// ´claim:persistence:a-second-write-replaces-the-checkpoint-so-a-read-sees-only-the-later-state´
/// ´test:crate:checkpoint-overwrite-existing´
#[test]
fn checkpoint_overwrite_existing() {
    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let working1 = test_working_copy();
    let payload1 = working1.to_checkpoint_payload(PersistentTimestamp::now(), Vec::new(), Vec::new());
    checkpoint::write_checkpoint(&path, &payload1).expect("first write failed");

    let mut working2 = test_working_copy();
    working2.last_processed_label_seq = 42;
    let payload2 = working2.to_checkpoint_payload(PersistentTimestamp::now(), Vec::new(), Vec::new());
    checkpoint::write_checkpoint(&path, &payload2).expect("second write failed");

    let restored = checkpoint::read_checkpoint(&path).expect("read failed");
    assert_eq!(restored.last_processed_label_seq, 42, "should read the second checkpoint");
}

/// An instance where no outcome axis has been registered checkpoints and
/// restores as having none — emptiness is recorded, not treated as missing
/// data. A host that checkpoints before registering its axes must come back
/// with an empty axis set rather than with a corrupt or absent one.
///
/// ´claim:persistence:an-instance-with-no-outcome-axes-round-trips-as-having-none´
/// ´test:crate:checkpoint-empty-outcome-models´
#[test]
fn checkpoint_empty_outcome_models() {
    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let working = test_working_copy(); // outcome_models is empty at cold start
    assert!(working.outcome_models.is_empty());

    let payload = test_payload(&working);
    checkpoint::write_checkpoint(&path, &payload).expect("write failed");
    let restored = checkpoint::read_checkpoint(&path).expect("read failed");

    assert!(restored.outcome_models.is_empty(), "empty outcome models should round-trip");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Journal Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A long run of appended records reads back complete and in the order it was
/// written, each entry still paired with the assessment it was about. The
/// length-prefixed framing has to survive being applied fifty times in
/// succession, because replay walks the file from the front and any record
/// whose frame is misread takes every record after it with it.
///
/// ´claim:persistence:a-run-of-appended-records-reads-back-complete-and-in-order´
/// ´test:crate:journal-50-entries´
#[test]
fn journal_50_entries() {
    let dir = test_dir();
    let path = dir.path().join("journal.bin");

    {
        let mut file = journal::open_journal_for_append(&path).unwrap();
        for i in 1..=50 {
            let entry = test_journal_entry(i);
            journal::append_journal_entry(&mut file, &entry).unwrap();
        }
    }

    let entries = journal::read_journal(&path).unwrap();
    assert_eq!(entries.len(), 50);

    for (i, entry) in entries.iter().enumerate() {
        assert_eq!(entry.seq, (i + 1) as u64);
        assert_eq!(entry.label.assessment_id, AssessmentId((i + 1) as u64));
    }
}

/// A journal whose final record was cut short — the shape a crash mid-write
/// leaves behind — gives up exactly that record and returns every complete one
/// before it. The last label the process was accepting when it died is the one
/// label it is allowed to lose; losing the other forty-nine along with it would
/// make the journal worthless precisely when it is needed.
///
/// ´claim:persistence:a-record-truncated-by-a-crash-costs-only-itself´
/// ´test:crate:journal-truncated-last-entry´
#[test]
fn journal_truncated_last_entry() {
    let dir = test_dir();
    let path = dir.path().join("journal.bin");

    {
        let mut file = journal::open_journal_for_append(&path).unwrap();
        for i in 1..=50 {
            let entry = test_journal_entry(i);
            journal::append_journal_entry(&mut file, &entry).unwrap();
        }
    }

    // Truncate the last few bytes to simulate a crash mid-write.
    let mut data = fs::read(&path).unwrap();
    let original_len = data.len();
    data.truncate(original_len - 5);
    fs::write(&path, &data).unwrap();

    let entries = journal::read_journal(&path).unwrap();
    assert_eq!(entries.len(), 49, "should return 49 complete entries");
    assert_eq!(entries.last().unwrap().seq, 49);
}

/// A journal file that exists but holds nothing reads as no entries at all
/// rather than as an error. Having nothing to replay is the ordinary case
/// after a clean checkpoint, and it must not be reported as a fault that
/// pushes the caller into a cold start.
///
/// ´claim:persistence:having-nothing-to-replay-reads-as-no-entries-not-as-a-failure´
/// ´test:crate:journal-empty-file´
#[test]
fn journal_empty_file() {
    let dir = test_dir();
    let path = dir.path().join("journal.bin");
    fs::write(&path, b"").unwrap();

    let entries = journal::read_journal(&path).unwrap();
    assert!(entries.is_empty());
}

/// A journal path that names no file at all is likewise no entries rather than
/// a failure. First start on a fresh deployment reaches the reader before
/// anything has ever written to that path, and it must be indistinguishable
/// from starting with an empty one.
///
/// (´claim:persistence:having-nothing-to-replay-reads-as-no-entries-not-as-a-failure´)
/// ´test:crate:journal-nonexistent-file´
#[test]
fn journal_nonexistent_file() {
    let dir = test_dir();
    let path = dir.path().join("nonexistent.bin");

    let entries = journal::read_journal(&path).unwrap();
    assert!(entries.is_empty());
}

/// Truncating at a mark the whole journal sits under empties it in both
/// senses: the file drops to its bare format header and a subsequent read
/// yields nothing. This is what a successful checkpoint does when every
/// journalled entry has been absorbed — anything at or below the mark left
/// behind would be replayed a second time on top of a model that already
/// contains it.
///
/// ´claim:persistence:truncation-empties-the-journal-both-on-disk-and-on-read´
/// ´test:crate:journal-truncate-resets-to-zero´
#[test]
fn journal_truncate_resets_to_zero() {
    let dir = test_dir();
    let path = dir.path().join("journal.bin");

    {
        let mut file = journal::open_journal_for_append(&path).unwrap();
        for i in 1..=10 {
            journal::append_journal_entry(&mut file, &test_journal_entry(i)).unwrap();
        }
    }

    assert!(fs::metadata(&path).unwrap().len() > 8);

    let mut writer = journal::JournalWriter::open(&path).unwrap();
    writer.truncate_to(10).unwrap();
    assert_eq!(
        fs::metadata(&path).unwrap().len(),
        8,
        "only the magic + version header remains"
    );

    let entries = journal::read_journal(&path).unwrap();
    assert!(entries.is_empty());
}

/// A journal of another format version is refused on the version and then
/// discarded by design, exactly as a checkpoint is: the reader reports the
/// mismatch rather than misreading records of another layout, and a writer
/// opened over the mismatched file starts a fresh journal numbered from the
/// checkpoint's mark. Construction survives the encounter — an upgraded
/// binary meeting the previous deployment's journal cold-starts, it does not
/// refuse to build (´dec:durability:checkpoint-journal´).
///
/// ´claim:persistence:a-journal-of-another-format-version-is-refused-then-discarded-by-design´
/// ´test:crate:journal-version-mismatch-discards-by-design´
#[test]
fn journal_version_mismatch_discards_by_design() {
    let dir = test_dir();
    let path = dir.path().join("journal.bin");

    // A headerless file — the shape the pre-version layout leaves on
    // disk — is refused on the version, not reported as corruption.
    fs::write(&path, [0x2A_u8; 64]).unwrap();
    match journal::read_journal(&path) {
        Err(journal::JournalError::VersionMismatch { found: None, expected }) => {
            assert_eq!(expected, 4);
        }
        other => panic!("expected VersionMismatch for a headerless file, got {other:?}"),
    }

    // A wrong-version header is refused carrying the version it declared.
    let mut data = Vec::new();
    data.extend_from_slice(b"ASAJ");
    data.extend_from_slice(&999_u32.to_le_bytes());
    fs::write(&path, &data).unwrap();
    match journal::read_journal(&path) {
        Err(journal::JournalError::VersionMismatch {
            found: Some(found),
            expected,
        }) => {
            assert_eq!(found, 999);
            assert_eq!(expected, 4);
        }
        other => panic!("expected VersionMismatch for version 999, got {other:?}"),
    }

    // An earlier generation's journal — version 2, before the context
    // gained the spatial-axis order its extractions were laid out against —
    // is likewise refused on the version, never parsed as the current
    // layout. Parsing it would be worse than refusing it: the pairs in those
    // extractions are ordered by a list that generation did not store.
    let mut prior = Vec::new();
    prior.extend_from_slice(b"ASAJ");
    prior.extend_from_slice(&2_u32.to_le_bytes());
    fs::write(&path, &prior).unwrap();
    match journal::read_journal(&path) {
        Err(journal::JournalError::VersionMismatch {
            found: Some(found),
            expected,
        }) => {
            assert_eq!(found, 2);
            assert_eq!(expected, 4);
        }
        other => panic!("expected VersionMismatch for the earlier version 2, got {other:?}"),
    }

    // The writer discards the mismatched file and numbers above the
    // checkpoint's mark, so the build proceeds where an error would
    // have refused it.
    let writer = journal::JournalWriter::open_after(&path, 500).unwrap();
    assert_eq!(writer.next_seq(), 501);
    let entries = journal::read_journal(&path).unwrap();
    assert!(entries.is_empty(), "the discarded journal reads as fresh");
}

/// Truncation measures against the checkpoint's mark, not against the file:
/// of five journalled entries with three absorbed, the two beyond the mark
/// survive the truncation intact and the writer keeps numbering above them.
/// A label appended after the checkpoint's capture is in the journal and not
/// in the checkpoint, so a truncation that discarded the whole file would
/// leave it in neither artefact — the silent loss
/// (´dec:durability:checkpoint-journal´) exists to make impossible.
///
/// ´claim:persistence:truncation-keeps-the-entries-the-checkpoint-has-not-absorbed´
/// ´test:crate:journal-truncate-to-retains-unabsorbed-tail´
#[test]
fn journal_truncate_to_retains_unabsorbed_tail() {
    let dir = test_dir();
    let path = dir.path().join("journal.bin");

    {
        let mut file = journal::open_journal_for_append(&path).unwrap();
        for i in 1..=5 {
            journal::append_journal_entry(&mut file, &test_journal_entry(i)).unwrap();
        }
    }

    let mut writer = journal::JournalWriter::open(&path).unwrap();
    assert_eq!(writer.next_seq(), 6);
    writer.truncate_to(3).unwrap();

    let entries = journal::read_journal(&path).unwrap();
    let seqs: Vec<u64> = entries.iter().map(|e| e.seq).collect();
    assert_eq!(seqs, [4, 5], "entries beyond the mark survive, in order");
    assert_eq!(
        entries[0].label.assessment_id,
        AssessmentId(4),
        "the surviving entry is the one that was appended, not a renumbering"
    );

    // The writer keeps appending above everything retained.
    assert_eq!(writer.next_seq(), 6);
    let appended = writer.append(&test_journal_entry(0)).unwrap();
    assert_eq!(appended, 6);
    let entries = journal::read_journal(&path).unwrap();
    let seqs: Vec<u64> = entries.iter().map(|e| e.seq).collect();
    assert_eq!(seqs, [4, 5, 6]);
}

/// One record alone is a valid journal: appended to a file that did not
/// previously exist, it reads back as exactly one entry with its sequence and
/// assessment intact. The framing does not depend on there being a following
/// record to bound it.
///
/// (´claim:persistence:a-run-of-appended-records-reads-back-complete-and-in-order´)
/// ´test:crate:journal-append-one-entry´
#[test]
fn journal_append_one_entry() {
    let dir = test_dir();
    let path = dir.path().join("journal.bin");

    {
        let mut file = journal::open_journal_for_append(&path).unwrap();
        journal::append_journal_entry(&mut file, &test_journal_entry(1)).unwrap();
    }

    let entries = journal::read_journal(&path).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].seq, 1);
    assert_eq!(entries[0].label.assessment_id, AssessmentId(1));
}

/// A label crosses the journal with every field as submitted: the assessment
/// it names, the action the host actually took, a negative valence carried with
/// its sign, and a ground-truth flag that stays false because it was never
/// raised. Replay re-applies these values as the update, so a field flipped or
/// defaulted in transit would teach the restored model something the host never
/// said.
///
/// ´claim:persistence:a-label-crosses-the-journal-with-every-field-as-submitted´
/// ´test:crate:journal-entry-fields-round-trip´
#[test]
fn journal_entry_fields_round_trip() {
    let dir = test_dir();
    let path = dir.path().join("journal.bin");

    let entry = JournalEntry {
        seq: 42,
        label: LabelSpec::new(AssessmentId(42)).action(Action::Block).valence(-1.0).build(),
        context: test_pending_context(AssessmentId(42)),
        timestamp: PersistentTimestamp::now(),
    };

    {
        let mut file = journal::open_journal_for_append(&path).unwrap();
        journal::append_journal_entry(&mut file, &entry).unwrap();
    }

    let entries = journal::read_journal(&path).unwrap();
    assert_eq!(entries.len(), 1);
    let restored = &entries[0];
    assert_eq!(restored.seq, 42);
    assert_eq!(restored.label.assessment_id, AssessmentId(42));
    assert_eq!(restored.label.action_taken, Action::Block);
    assert_eq!(restored.label.valence, -1.0);
    assert!(!restored.label.ground_truth);
}

/// Per-axis outcomes survive the journal still attached to the axes they were
/// reported against, including axis identifiers far outside any contiguous
/// range and values of either sign. The axis keys are what route each value to
/// its own model on replay, so a mapping scrambled in transit would credit one
/// axis with another's evidence.
///
/// ´claim:persistence:per-axis-outcomes-survive-the-journal-still-attached-to-their-axes´
/// ´test:crate:journal-entry-with-outcomes´
#[test]
fn journal_entry_with_outcomes() {
    let dir = test_dir();
    let path = dir.path().join("journal.bin");

    let mut outcomes = HashMap::new();
    outcomes.insert(OutcomeAxisId(1), 0.75);
    outcomes.insert(OutcomeAxisId(2), -0.5);
    outcomes.insert(OutcomeAxisId(99), 1.0);

    let mut spec = LabelSpec::new(AssessmentId(7)).valence(1.0).ground_truth();
    for (axis, value) in &outcomes {
        spec = spec.outcome(*axis, *value);
    }
    let entry = JournalEntry {
        seq: 7,
        label: spec.build(),
        context: test_pending_context(AssessmentId(7)),
        timestamp: PersistentTimestamp::now(),
    };

    {
        let mut file = journal::open_journal_for_append(&path).unwrap();
        journal::append_journal_entry(&mut file, &entry).unwrap();
    }

    let entries = journal::read_journal(&path).unwrap();
    assert_eq!(entries.len(), 1);
    let outcomes = &entries[0].label.outcomes;
    assert_eq!(outcomes.len(), 3);
    assert_eq!(outcomes[&OutcomeAxisId(1)], 0.75);
    assert_eq!(outcomes[&OutcomeAxisId(2)], -0.5);
    assert_eq!(outcomes[&OutcomeAxisId(99)], 1.0);
}

/// The journal records what was submitted, not what was unique: a label
/// journaled twice for the same assessment — the shape a host retry leaves
/// after a journalled-but-not-enqueued result — appears twice, under distinct
/// sequence numbers, with the unrelated label between them untouched. Writing
/// is on the hot path and cannot afford to scan history for duplicates;
/// deciding which attempt counts is deferred to replay, where the whole file
/// is in hand anyway.
///
/// ´claim:persistence:the-journal-records-repeated-attempts-and-defers-deduplication-to-replay´
/// ´test:crate:journal-duplicate-assessment-id-allowed´
#[test]
fn journal_duplicate_assessment_id_allowed() {
    let dir = test_dir();
    let path = dir.path().join("journal.bin");

    // Write two entries with the same assessment_id (simulating retry scenario).
    {
        let mut file = journal::open_journal_for_append(&path).unwrap();

        // First attempt: journaled successfully.
        let entry1 = JournalEntry {
            seq: 1,
            label: test_label(AssessmentId(42)), // assessment_id = 42
            context: test_pending_context(AssessmentId(42)),
            timestamp: PersistentTimestamp::now(),
        };
        journal::append_journal_entry(&mut file, &entry1).unwrap();

        // Second attempt: host retried after JournaledButNotEnqueued.
        let entry2 = JournalEntry {
            seq: 2,
            label: test_label(AssessmentId(42)), // same assessment_id = 42
            context: test_pending_context(AssessmentId(42)),
            timestamp: PersistentTimestamp::now(),
        };
        journal::append_journal_entry(&mut file, &entry2).unwrap();

        // Third entry: different assessment_id.
        let entry3 = JournalEntry {
            seq: 3,
            label: test_label(AssessmentId(43)),
            context: test_pending_context(AssessmentId(43)),
            timestamp: PersistentTimestamp::now(),
        };
        journal::append_journal_entry(&mut file, &entry3).unwrap();
    }

    // Journal should contain all 3 entries (dedup is at replay time, not here).
    let entries = journal::read_journal(&path).unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].label.assessment_id, AssessmentId(42));
    assert_eq!(entries[1].label.assessment_id, AssessmentId(42)); // duplicate allowed
    assert_eq!(entries[2].label.assessment_id, AssessmentId(43));
}

/// A record carrying a thousand signal features round-trips with its full
/// length and with an interior value still where it was put. The length prefix
/// is a fixed-width field ahead of a variable payload, so the interesting
/// question is whether a record much larger than the frame that describes it is
/// still bounded correctly — a wide feature vector is an ordinary
/// configuration, not an edge case.
///
/// ´claim:persistence:a-record-far-larger-than-its-frame-is-bounded-and-read-back-whole´
/// ´test:crate:journal-large-context´
#[test]
fn journal_large_context() {
    let dir = test_dir();
    let path = dir.path().join("journal.bin");

    // Create a context with 1000-element signal features.
    let mut context = test_pending_context(AssessmentId(1));
    context.signal_features = (0..1000).map(|i| i as f32 * 0.001).collect::<Vec<f32>>().into();

    let entry = JournalEntry {
        seq: 1,
        label: test_label(AssessmentId(1)),
        context,
        timestamp: PersistentTimestamp::now(),
    };

    {
        let mut file = journal::open_journal_for_append(&path).unwrap();
        journal::append_journal_entry(&mut file, &entry).unwrap();
    }

    let entries = journal::read_journal(&path).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].context.signal_features.len(), 1000);
    assert!((entries[0].context.signal_features.value(500) - 0.5).abs() < 1e-6);
}

/// Each Sentinel's extraction survives the journal as its own triple —
/// coordinate, features and occupancy — keyed by the Sentinel it came from,
/// with an unoccupied extraction staying unoccupied. Replay routes each
/// feature set back into its own Sentinel's graph at that coordinate, and the
/// occupancy flag is what distinguishes a Sentinel that had nothing to say from
/// one that said zero.
///
/// ´claim:persistence:each-sentinels-extraction-survives-the-journal-under-its-own-identifier´
/// ´test:crate:journal-sentinel-extractions-round-trip´
#[test]
fn journal_sentinel_extractions_round_trip() {
    let dir = test_dir();
    let path = dir.path().join("journal.bin");

    let mut context = test_pending_context(AssessmentId(1));
    context
        .sentinel_extractions
        .insert(SentinelId(1), SentinelExtraction::new(12345, vec![0.1, 0.2, 0.3], true));
    context
        .sentinel_extractions
        .insert(SentinelId(2), SentinelExtraction::new(67890, vec![-0.5, 0.5], false));

    let entry = JournalEntry {
        seq: 1,
        label: test_label(AssessmentId(1)),
        context,
        timestamp: PersistentTimestamp::now(),
    };

    {
        let mut file = journal::open_journal_for_append(&path).unwrap();
        journal::append_journal_entry(&mut file, &entry).unwrap();
    }

    let entries = journal::read_journal(&path).unwrap();
    assert_eq!(entries.len(), 1);

    let extractions = &entries[0].context.sentinel_extractions;
    assert_eq!(extractions.len(), 2);

    let s1 = &extractions[&SentinelId(1)];
    assert_eq!(s1.coordinate, 12345);
    assert_eq!(s1.features, crate::pending::StoredFeatures::Single(vec![0.1, 0.2, 0.3]));
    assert!(s1.occupancy);

    let s2 = &extractions[&SentinelId(2)];
    assert_eq!(s2.coordinate, 67890);
    assert_eq!(s2.features, crate::pending::StoredFeatures::Single(vec![-0.5, 0.5]));
    assert!(!s2.occupancy);
}

// ═══════════════════════════════════════════════════════════════════════════════
// JournalWriter Sequence Number Tests (´dec:durability:checkpoint-journal´)
// ═══════════════════════════════════════════════════════════════════════════════

/// The writer, not the caller, decides sequence numbers: entries handed to it
/// carrying nothing meaningful come out numbered one, two, three, and the file
/// on disk agrees with the numbers it returned. Sequence is the ordering
/// replay depends on and the coordinate the checkpoint's high-water mark is
/// expressed in, so it can only be assigned at the single point where writes
/// are serialised.
///
/// ´claim:persistence:the-writer-assigns-sequence-numbers-itself-overriding-what-the-caller-supplied´
/// ´test:crate:journal-writer-assigns-monotonic-seq´
#[test]
fn journal_writer_assigns_monotonic_seq() {
    let dir = test_dir();
    let path = dir.path().join("journal.bin");

    let mut writer = journal::JournalWriter::open(&path).unwrap();
    let seq1 = writer.append(&test_journal_entry(0)).unwrap();
    let seq2 = writer.append(&test_journal_entry(0)).unwrap();
    let seq3 = writer.append(&test_journal_entry(0)).unwrap();

    assert_eq!(seq1, 1);
    assert_eq!(seq2, 2);
    assert_eq!(seq3, 3);

    drop(writer);
    let entries = journal::read_journal(&path).unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].seq, 1);
    assert_eq!(entries[1].seq, 2);
    assert_eq!(entries[2].seq, 3);
}

/// A writer opened over a journal that already holds entries reads the file
/// before writing to it and resumes above the highest sequence it finds. A
/// restarted process appends to the same journal the previous one left, and
/// numbering from scratch would produce two records claiming the same position
/// in a single replay ordering.
///
/// ´claim:persistence:a-reopened-writer-resumes-above-the-highest-sequence-already-on-disk´
/// ´test:crate:journal-writer-resumes-after-existing-entries´
#[test]
fn journal_writer_resumes_after_existing_entries() {
    let dir = test_dir();
    let path = dir.path().join("journal.bin");

    // Write 5 entries with raw append (known seq values).
    {
        let mut file = journal::open_journal_for_append(&path).unwrap();
        for i in 1..=5 {
            journal::append_journal_entry(&mut file, &test_journal_entry(i)).unwrap();
        }
    }

    // Open JournalWriter — should resume at seq 6.
    let mut writer = journal::JournalWriter::open(&path).unwrap();
    assert_eq!(writer.next_seq(), 6);

    let seq = writer.append(&test_journal_entry(0)).unwrap();
    assert_eq!(seq, 6);

    drop(writer);
    let entries = journal::read_journal(&path).unwrap();
    assert_eq!(entries.len(), 6);
    assert_eq!(entries[5].seq, 6);
}

/// Resuming after a restore takes the higher of the two marks it has: the
/// checkpoint's last processed sequence wins over a journal that only reaches
/// a lower one. The two can disagree in either direction — a journal truncated
/// after checkpointing lags, an unabsorbed tail leads — and only the maximum
/// guarantees the next number is above everything the model has already seen.
///
/// ´claim:persistence:resuming-takes-the-higher-of-the-checkpoint-and-journal-marks´
/// ´test:crate:journal-writer-open-after-uses-max-of-checkpoint-and-journal´
#[test]
fn journal_writer_open_after_uses_max_of_checkpoint_and_journal() {
    let dir = test_dir();
    let path = dir.path().join("journal.bin");

    // Write entries with seq 1–3.
    {
        let mut file = journal::open_journal_for_append(&path).unwrap();
        for i in 1..=3 {
            journal::append_journal_entry(&mut file, &test_journal_entry(i)).unwrap();
        }
    }

    // open_after with last_processed_seq=10 (higher than journal max).
    let mut writer = journal::JournalWriter::open_after(&path, 10).unwrap();
    assert_eq!(writer.next_seq(), 11);

    let seq = writer.append(&test_journal_entry(0)).unwrap();
    assert_eq!(seq, 11);
}

/// Numbering begins at one on a journal that has never been written, leaving
/// zero free to mean unsequenced. An entry still carrying zero has not been
/// through the writer, and that distinction is only available if no real record
/// ever occupies the value.
///
/// ´claim:persistence:numbering-begins-at-one-leaving-zero-to-mean-unsequenced´
/// ´test:crate:journal-writer-open-empty-starts-at-one´
#[test]
fn journal_writer_open_empty_starts_at_one() {
    let dir = test_dir();
    let path = dir.path().join("journal.bin");

    let writer = journal::JournalWriter::open(&path).unwrap();
    assert_eq!(writer.next_seq(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Replay Deduplication Tests (´cor:durability:replay-exactness´)
// ═══════════════════════════════════════════════════════════════════════════════

/// Replay applies each assessment once however often it was journaled: five
/// records naming four distinct assessments produce four updates and one
/// deliberate skip. Deduplication is keyed on the assessment rather than on the
/// sequence number, because the retry that produced the second record was a
/// second attempt at the same event, not a second event.
///
/// ´claim:persistence:replay-applies-each-assessment-once-however-often-it-was-journaled´
/// ´test:crate:replay-dedup-filters-duplicate-assessment-id´
#[test]
fn replay_dedup_filters_duplicate_assessment_id() {
    // Simulate journal entries with duplicates (as returned by recovery).
    let entries = vec![
        test_journal_entry_with_assessment_id(6, AssessmentId(100)), // assessment_id=100
        test_journal_entry_with_assessment_id(7, AssessmentId(101)), // assessment_id=101
        test_journal_entry_with_assessment_id(8, AssessmentId(100)), // duplicate assessment_id=100
        test_journal_entry_with_assessment_id(9, AssessmentId(102)), // assessment_id=102
        test_journal_entry_with_assessment_id(10, AssessmentId(103)), // assessment_id=103
    ];

    // Apply the deduplication logic from lib.rs.
    let mut seen_ids: HashSet<AssessmentId> = HashSet::new();
    let mut replayed = 0;
    let mut skipped_duplicate = 0;

    for entry in &entries {
        if !seen_ids.insert(entry.label.assessment_id) {
            skipped_duplicate += 1;
            continue;
        }
        replayed += 1;
    }

    assert_eq!(replayed, 4, "should replay 4 unique entries");
    assert_eq!(skipped_duplicate, 1, "should skip 1 duplicate");
}

/// Of two records for one assessment it is the earlier that is kept: replay
/// walks in sequence order and the survivor is the one with the lower number.
/// Keeping the later record instead would be a different and worse rule — the
/// first attempt is the one the model may already have partly seen, and
/// preferring the retry would make replay's result depend on how many times the
/// host happened to retry.
///
/// ´claim:persistence:where-attempts-repeat-it-is-the-earliest-that-replay-keeps´
/// ´test:crate:replay-dedup-preserves-first-occurrence´
#[test]
fn replay_dedup_preserves_first_occurrence() {
    let entries = vec![
        test_journal_entry_with_assessment_id(1, AssessmentId(42)), // first occurrence
        test_journal_entry_with_assessment_id(2, AssessmentId(42)), // second occurrence (skipped)
    ];

    let mut seen_ids: HashSet<AssessmentId> = HashSet::new();
    let mut kept_seqs: Vec<u64> = Vec::new();

    for entry in &entries {
        if !seen_ids.insert(entry.label.assessment_id) {
            continue;
        }
        kept_seqs.push(entry.seq);
    }

    assert_eq!(kept_seqs, vec![1], "should keep only seq=1 (first occurrence)");
}

/// Nothing to replay produces no updates: the dedup pass over an empty set of
/// entries leaves the restored model exactly as the checkpoint described it.
/// This is the common path after a clean shutdown, and it must not touch the
/// model on the way past.
///
/// ´claim:persistence:nothing-to-replay-leaves-the-restored-model-as-the-checkpoint-described-it´
/// ´test:crate:replay-dedup-empty-input´
#[test]
fn replay_dedup_empty_input() {
    let entries: Vec<JournalEntry> = vec![];

    let mut seen_ids: HashSet<AssessmentId> = HashSet::new();
    let mut replayed = 0;

    for entry in &entries {
        if !seen_ids.insert(entry.label.assessment_id) {
            continue;
        }
        replayed += 1;
    }

    assert_eq!(replayed, 0);
}

/// Helper: creates a journal entry with explicit seq and assessment_id.
fn test_journal_entry_with_assessment_id(seq: u64, assessment_id: AssessmentId) -> JournalEntry {
    JournalEntry {
        seq,
        label: test_label(assessment_id),
        context: test_pending_context(assessment_id),
        timestamp: PersistentTimestamp::now(),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Recovery Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Every checkpoint field the restore's structural validation reads is a field
/// the checkpoint schema declares, with neither side of the comparison written
/// down: the schema comes from the payload's own serialisation and the
/// validated fields from the validation function's own body, bounded at its
/// closing brace. The correspondence is then put under drift in both directions
/// it can take — a schema that has lost a validated field is refused, once for
/// each field the discovery found and each refusal naming the field it lost,
/// and a validation that has gained a field the schema never declared is
/// refused likewise. Two surfaces written in different places are what
/// (´cav:durability:restore-field-mismatch´) says nothing makes agree; a test
/// holding its own copy of either would keep passing through exactly the edit
/// it exists to catch, so it holds neither.
///
/// ´claim:persistence:every-restore-validated-field-belongs-to-the-checkpoint-schema´
/// ´test:crate:restore-validation-fields-belong-to-checkpoint-schema´
#[test]
fn restore_validation_fields_belong_to_checkpoint_schema() {
    let checkpoint_fields = checkpoint_schema_fields(&test_payload(&test_working_copy()));
    let validated_fields = restore_validation_fields();

    // Non-vacuity: the discovery found reads on the validation side, and the
    // two sides are genuinely different surfaces rather than one surface read
    // twice. Were the schema set to collapse onto the validated set, the
    // correspondence below would hold for a reason that has nothing to do with
    // the checkpoint.
    assert!(!validated_fields.is_empty(), "restore validation reads checkpoint fields");
    assert!(
        checkpoint_fields.difference(&validated_fields).next().is_some(),
        "the checkpoint declares fields the restore validation does not read: \
         schema {checkpoint_fields:?}, validated {validated_fields:?}"
    );

    // The obligation itself, against the live pair.
    require_declared_restore_fields(&checkpoint_fields, &validated_fields)
        .expect("the live checkpoint schema declares every restore-validated field");

    // Drift, first direction: a checkpoint field that goes away. Every field
    // the discovery found is individually load-bearing — dropping any one of
    // them from the schema makes this same oracle refuse and name it — so no
    // discovered field is carried along unchecked by the others.
    for lost in &validated_fields {
        let mut lost_a_field = checkpoint_fields.clone();
        assert!(lost_a_field.remove(lost), "the ablated field was declared");
        let refusal = require_declared_restore_fields(&lost_a_field, &validated_fields)
            .expect_err("a schema that lost a validated field is refused");
        assert!(refusal.contains(lost.as_str()), "the refusal names the lost field: {refusal}");
    }

    // Drift, second direction: a validation that starts reading a field the
    // schema never declared — the failure mode the caveat records.
    let fabricated = "no_such_checkpoint_field";
    assert!(
        !checkpoint_fields.contains(fabricated),
        "the fabricated field is undeclared: {checkpoint_fields:?}"
    );
    let mut gained_a_read = validated_fields;
    gained_a_read.insert(fabricated.to_owned());
    let refusal = require_declared_restore_fields(&checkpoint_fields, &gained_a_read)
        .expect_err("a validation reading an undeclared field is refused");
    assert!(
        refusal.contains(fabricated),
        "the refusal names the undeclared field: {refusal}"
    );
}

/// Restoring a day-old checkpoint succeeds and hands back a working copy at
/// the dimensions it was saved with — the operational model at its full width,
/// the anchor at its own narrower one. Ageing the state for the downtime
/// happens on the way through, and it rescales what the models hold rather than
/// reshaping them.
///
/// ´claim:persistence:restore-returns-a-working-copy-at-the-dimensions-it-was-saved-with´
/// ´test:crate:recovery-with-time-decay´
#[test]
fn recovery_with_time_decay() {
    let dir = test_dir();
    let cp_path = dir.path().join("checkpoint.bin");
    let jl_path = dir.path().join("journal.bin");

    let working = test_working_copy();

    // Create a checkpoint timestamped 24 hours ago.
    let now_secs = PersistentTimestamp::now().seconds;
    let past = PersistentTimestamp::new(now_secs - 24 * 3600, 0);
    let payload = working.to_checkpoint_payload(past, Vec::new(), Vec::new());
    checkpoint::write_checkpoint(&cp_path, &payload).expect("write failed");

    // Empty journal.
    fs::write(&jl_path, b"").unwrap();

    let config = test_persistence_config(None);

    let result = recovery::attempt_restore(&cp_path, &jl_path, &config, &[], &[], PersistentTimestamp::now())
        .expect("the fixture checkpoint is not refused on its numbers");
    assert!(result.is_some(), "recovery should succeed");

    let result = result.unwrap();
    assert!(result.replay_entries.is_empty());

    // After 24 hours of decay with γ_t_core=0.9999, the factor is ~0.9976.
    // Precision should have been scaled down (and covariance up).
    // Just verify the dimensions are correct — the actual values
    // are tested by the model's apply_time_decay unit test.
    assert_eq!(result.working.operational.dim(), 16);
    assert_eq!(result.working.anchor.dim(), 15);
}

/// Recovery hands back only the journal entries the checkpoint had not already
/// absorbed: against a snapshot taken at the fifth label, a journal holding ten
/// yields the last five, starting exactly one past the mark. The checkpoint
/// already contains the effect of everything up to its own sequence, so
/// replaying those records would apply the same evidence a second time.
///
/// ´claim:persistence:only-entries-beyond-the-checkpoints-mark-are-handed-back-for-replay´
/// ´test:crate:recovery-with-journal-replay´
#[test]
fn recovery_with_journal_replay() {
    let dir = test_dir();
    let cp_path = dir.path().join("checkpoint.bin");
    let jl_path = dir.path().join("journal.bin");

    let mut working = test_working_copy();
    working.last_processed_label_seq = 5;

    let payload = working.to_checkpoint_payload(PersistentTimestamp::now(), Vec::new(), Vec::new());
    checkpoint::write_checkpoint(&cp_path, &payload).expect("write failed");

    // Write journal entries 1–10 (entries 1–5 should be skipped).
    {
        let mut file = journal::open_journal_for_append(&jl_path).unwrap();
        for i in 1..=10 {
            journal::append_journal_entry(&mut file, &test_journal_entry(i)).unwrap();
        }
    }

    let config = test_persistence_config(None);

    let result = recovery::attempt_restore(&cp_path, &jl_path, &config, &[], &[], PersistentTimestamp::now())
        .expect("the fixture checkpoint is not refused on its numbers")
        .unwrap();

    // Entries 1–5 should be filtered out; only 6–10 remain.
    assert_eq!(result.replay_entries.len(), 5);
    assert_eq!(result.replay_entries[0].seq, 6);
    assert_eq!(result.replay_entries[4].seq, 10);
}

/// A checkpoint whose anchor model has a different width than this build uses
/// is refused, even though the file itself is perfectly intact and passes every
/// integrity check. Feature layout is fixed at construction, so state saved
/// under a different geometry describes coefficients for positions that no
/// longer mean what they meant; starting cold is the only honest option.
///
/// ´claim:persistence:state-saved-under-a-different-geometry-is-refused-despite-being-intact´
/// ´test:crate:recovery-structural-mismatch´
#[test]
fn recovery_structural_mismatch() {
    let dir = test_dir();
    let cp_path = dir.path().join("checkpoint.bin");
    let jl_path = dir.path().join("journal.bin");

    // Create a working copy then tamper with the checkpoint's anchor
    // dimension to simulate a binary from a different build where P_A
    // was different. The normal code path always produces P_A = 15.
    let working = test_working_copy();
    let mut payload = working.to_checkpoint_payload(PersistentTimestamp::now(), Vec::new(), Vec::new());
    // Fabricate a mismatched anchor dimension (overwrite p).
    payload.anchor.parameters.p = 8;
    payload.anchor.parameters.mu = vec![0.0; 8];
    let cov_size = 8 * (8 + 1) / 2;
    payload.anchor.parameters.covariance_data = vec![0.0; cov_size];
    checkpoint::write_checkpoint(&cp_path, &payload).expect("write failed");
    fs::write(&jl_path, b"").unwrap();

    let config = AssayerConfig::default();

    let result = recovery::attempt_restore(&cp_path, &jl_path, &config, &[], &[], PersistentTimestamp::now())
        .expect("the fixture checkpoint is not refused on its numbers");
    assert!(result.is_none(), "structural mismatch should return None");
}

/// Pointed at a checkpoint that does not exist, recovery declines rather than
/// failing: the caller is told there is nothing to restore from and starts
/// cold. Every first start on a fresh deployment takes this path, so an absent
/// checkpoint has to be an ordinary answer rather than an error condition.
///
/// ´claim:persistence:an-unusable-checkpoint-yields-a-cold-start-rather-than-a-refusal-to-run´
/// ´test:crate:recovery-missing-checkpoint´
#[test]
fn recovery_missing_checkpoint() {
    let dir = test_dir();
    let cp_path = dir.path().join("nonexistent.bin");
    let jl_path = dir.path().join("journal.bin");

    let config = AssayerConfig::default();
    let result = recovery::attempt_restore(&cp_path, &jl_path, &config, &[], &[], PersistentTimestamp::now())
        .expect("the fixture checkpoint is not refused on its numbers");
    assert!(result.is_none(), "missing checkpoint should return None");
}

/// A file at the checkpoint path that is simply not a checkpoint — arbitrary
/// bytes, no header, no structure — leaves recovery declining rather than
/// panicking or half-restoring. Losing the learned state to a cold start is
/// recoverable in a way that a process refusing to come up is not.
///
/// (´claim:persistence:an-unusable-checkpoint-yields-a-cold-start-rather-than-a-refusal-to-run´)
/// ´test:crate:recovery-corrupt-checkpoint´
#[test]
fn recovery_corrupt_checkpoint() {
    let dir = test_dir();
    let cp_path = dir.path().join("checkpoint.bin");
    let jl_path = dir.path().join("journal.bin");

    // Write arbitrary garbage that is not a valid checkpoint.
    fs::write(&cp_path, b"this is not a valid checkpoint file at all").unwrap();
    fs::write(&jl_path, b"").unwrap();

    let config = AssayerConfig::default();
    let result = recovery::attempt_restore(&cp_path, &jl_path, &config, &[], &[], PersistentTimestamp::now())
        .expect("the fixture checkpoint is not refused on its numbers");
    assert!(result.is_none(), "corrupt checkpoint should return None");
}

/// The reader's version refusal reaches the caller as a cold start: a
/// checkpoint left behind by a binary speaking another format version stops
/// recovery without stopping the process. Deploying a version that changed the
/// layout is a routine event, and it must cost the learned state and nothing
/// more.
///
/// (´claim:persistence:an-unusable-checkpoint-yields-a-cold-start-rather-than-a-refusal-to-run´)
/// ´test:crate:recovery-version-mismatch-returns-none´
#[test]
fn recovery_version_mismatch_returns_none() {
    let dir = test_dir();
    let cp_path = dir.path().join("checkpoint.bin");
    let jl_path = dir.path().join("journal.bin");

    let working = test_working_copy();
    let payload = working.to_checkpoint_payload(PersistentTimestamp::now(), Vec::new(), Vec::new());
    checkpoint::write_checkpoint(&cp_path, &payload).expect("write failed");

    // Tamper with the version field (bytes 4..8) to simulate old format.
    let mut data = fs::read(&cp_path).unwrap();
    data[4] = 0xFF;
    data[5] = 0xFF;
    data[6] = 0xFF;
    data[7] = 0xFF;
    fs::write(&cp_path, &data).unwrap();
    fs::write(&jl_path, b"").unwrap();

    let config = test_persistence_config(None);

    let result = recovery::attempt_restore(&cp_path, &jl_path, &config, &[], &[], PersistentTimestamp::now())
        .expect("the fixture checkpoint is not refused on its numbers");
    assert!(result.is_none(), "version mismatch should return None");
}

/// The label high-water mark survives the whole restore path unchanged, not
/// merely the file format: the working copy handed back reports the same
/// sequence the checkpoint was taken at. That number is what the replay filter
/// and the resumed writer both measure against, so it is the one field whose
/// loss would silently double-apply or skip labels.
///
/// ´claim:persistence:the-label-high-water-mark-survives-the-whole-restore-path´
/// ´test:crate:recovery-preserves-last-processed-seq´
#[test]
fn recovery_preserves_last_processed_seq() {
    let dir = test_dir();
    let cp_path = dir.path().join("checkpoint.bin");
    let jl_path = dir.path().join("journal.bin");

    let mut working = test_working_copy();
    working.last_processed_label_seq = 77;

    let payload = working.to_checkpoint_payload(PersistentTimestamp::now(), Vec::new(), Vec::new());
    checkpoint::write_checkpoint(&cp_path, &payload).expect("write failed");
    fs::write(&jl_path, b"").unwrap();

    let config = test_persistence_config(None);

    let result = recovery::attempt_restore(&cp_path, &jl_path, &config, &[], &[], PersistentTimestamp::now())
        .expect("the fixture checkpoint is not refused on its numbers")
        .unwrap();
    assert_eq!(
        result.working.last_processed_label_seq, 77,
        "seq should be preserved through checkpoint restore"
    );
}

/// Downtime costs the model confidence in the right direction: after restoring
/// a day-old checkpoint the precision diagonal is lower than it was and the
/// covariance diagonal higher. Evidence gathered before an outage describes a
/// world that has since moved on, so the restored model must hold its beliefs
/// more loosely — not merely hold them.
///
/// ´claim:persistence:downtime-lowers-precision-and-widens-covariance-so-old-evidence-counts-for-less´
/// ´test:crate:recovery-decay-applied-to-precision´
#[test]
fn recovery_decay_applied_to_precision() {
    let dir = test_dir();
    let cp_path = dir.path().join("checkpoint.bin");
    let jl_path = dir.path().join("journal.bin");

    let working = test_working_copy();

    // Capture pre-checkpoint diagonal values for comparison.
    let pre_op_diag_0 = working.operational.precision().diagonal_element(0);

    // Create a checkpoint timestamped 24 hours ago.
    let now_secs = PersistentTimestamp::now().seconds;
    let past = PersistentTimestamp::new(now_secs - 24 * 3600, 0);
    let payload = working.to_checkpoint_payload(past, Vec::new(), Vec::new());
    checkpoint::write_checkpoint(&cp_path, &payload).expect("write failed");
    fs::write(&jl_path, b"").unwrap();

    let config = test_persistence_config(None);

    let result = recovery::attempt_restore(&cp_path, &jl_path, &config, &[], &[], PersistentTimestamp::now())
        .expect("the fixture checkpoint is not refused on its numbers")
        .unwrap();

    // After 24 hours of decay with γ_t_core=0.9999, factor ≈ 0.9976.
    // Precision should have been scaled down.
    let post_op_diag_0 = result.working.operational.precision().diagonal_element(0);
    assert!(
        post_op_diag_0 < pre_op_diag_0,
        "precision diagonal should decrease after time-decay: {post_op_diag_0} < {pre_op_diag_0}"
    );

    // Covariance should have been scaled up.
    let pre_cov_diag_0 =
        1.0 / working.operational.precision().diagonal_element(0) * working.operational.precision().diagonal_element(0);
    let post_cov_diag_0 = result.working.operational.covariance().diagonal_element(0);
    let pre_cov_diag_0_actual = working.operational.covariance().diagonal_element(0);
    assert!(
        post_cov_diag_0 > pre_cov_diag_0_actual,
        "covariance diagonal should increase after time-decay: {post_cov_diag_0} > {pre_cov_diag_0}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Scheduler Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// The scheduler's whole job is to ask: on each interval it puts a checkpoint
/// request on the command channel and nothing more. Writing the file is the
/// model owner's work, because only that thread holds the state consistently;
/// a scheduler that wrote checkpoints itself would have to take a lock across
/// the entire snapshot.
///
/// ´claim:persistence:the-scheduler-requests-checkpoints-on-its-interval-and-writes-none-itself´
/// ´test:crate:scheduler-sends-checkpoint-command´
#[test]
fn scheduler_sends_checkpoint_command() {
    let (command_tx, command_rx) = crossbeam_channel::bounded::<crate::owner::commands::ModelOwnerCommand>(64);
    let (control_tx, control_rx) = crossbeam_channel::bounded::<()>(0);

    let handle = scheduler::spawn_checkpoint_scheduler("test", Duration::from_millis(50), command_tx, control_rx);

    // Wait for at least one checkpoint command.
    let received = command_rx.recv_timeout(ACK_DEADLINE);
    assert!(received.is_ok(), "should receive checkpoint command");

    match received.unwrap() {
        crate::owner::commands::ModelOwnerCommand::Checkpoint(_) => {}
        other => panic!("expected Checkpoint, got {other:?}"),
    }

    // Shut down.
    drop(control_tx);
    handle.join().expect("scheduler thread panicked");
}

/// Dropping the control channel ends the scheduler promptly even when its
/// interval is an hour away: the thread is waiting on that channel with a
/// timeout, not sleeping through it, so disconnection is noticed at once. A
/// scheduler that only checked for shutdown when its timer fired would hold up
/// every shutdown by up to a full interval.
///
/// ´claim:persistence:dropping-the-control-channel-ends-the-scheduler-without-waiting-out-its-interval´
/// ´test:crate:scheduler-shutdown-on-drop´
#[test]
fn scheduler_shutdown_on_drop() {
    let (command_tx, _command_rx) = crossbeam_channel::bounded::<crate::owner::commands::ModelOwnerCommand>(64);
    let (control_tx, control_rx) = crossbeam_channel::bounded::<()>(0);

    let handle = scheduler::spawn_checkpoint_scheduler("test", Duration::from_secs(3600), command_tx, control_rx);

    // Drop the control sender to signal shutdown.
    drop(control_tx);

    // Thread should exit promptly.
    let join_result = handle.join();
    assert!(join_result.is_ok(), "scheduler thread should exit cleanly");
}

/// The scheduler thread carries its instance's name alongside its role. Where
/// several instances share a process, a thread dump or profile that named them
/// all identically would leave an operator unable to tell whose checkpointing
/// is stalling.
///
/// ´claim:persistence:the-scheduler-thread-is-named-for-its-instance-as-well-as-its-role´
/// ´test:crate:scheduler-thread-name´
#[test]
fn scheduler_thread_name() {
    let (command_tx, _command_rx) = crossbeam_channel::bounded::<crate::owner::commands::ModelOwnerCommand>(64);
    let (control_tx, control_rx) = crossbeam_channel::bounded::<()>(0);

    // Use a distinctive instance ID to verify the thread name.
    let handle = scheduler::spawn_checkpoint_scheduler("myinst-42", Duration::from_secs(3600), command_tx, control_rx);

    // The thread should be named "{instance_id}-checkpoint".
    let thread = handle.thread();
    assert_eq!(
        thread.name(),
        Some("myinst-42-checkpoint"),
        "scheduler thread name should follow convention"
    );

    drop(control_tx);
    handle.join().expect("scheduler thread panicked");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Integration Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Turning persistence on changes nothing about how an instance comes up: it
/// builds against an empty directory and publishes its first snapshot exactly
/// as a non-persistent one does. The extra thread and the extra files are not
/// a precondition for serving.
///
/// ´claim:persistence:enabling-persistence-does-not-change-how-an-instance-comes-up´
/// ´test:crate:assayer-with-persistence-constructs´
#[test]
fn assayer_with_persistence_constructs() {
    let dir = test_dir();

    let config = test_persistence_config(Some(dir.path()));

    let assayer = crate::Assayer::build(config).expect("assayer builds");

    // Verify it started.
    let guard = assayer.shared().published.load();
    assert_eq!(guard.version, 1);

    // Drop triggers orderly shutdown.
    drop(assayer);
}

/// Dropping a persistent instance returns rather than hanging. Shutdown has to
/// unwind two threads that hold channels to each other, and the ordering
/// matters: were the model owner to stop first, a scheduler still asking it for
/// checkpoints would keep the drop waiting forever.
///
/// ´claim:persistence:dropping-a-persistent-instance-unwinds-both-threads-without-hanging´
/// ´test:crate:assayer-drop-shuts-down-scheduler-first´
#[test]
fn assayer_drop_shuts_down_scheduler_first() {
    let dir = test_dir();

    let config = test_persistence_config(Some(dir.path()));

    let assayer = crate::Assayer::build(config).expect("assayer builds");
    // Just verify drop completes without hang or panic.
    drop(assayer);
}

/// A journal path that cannot be opened fails the construction rather than
/// aborting the process: the caller gets `PersistenceSetupFailed`, the variant
/// whose documentation describes this exact condition. Construction is the
/// package's fallible surface and an unwritable path is a caller-supplied
/// configuration the contract does not admit, so it belongs on the error
/// channel that already names it. The ordering matters as much as the channel
/// — the open is attempted before any thread is spawned, so a refused build
/// leaves nothing running behind it.
///
/// ´claim:persistence:an-unopenable-journal-refuses-the-build-rather-than-aborting-the-process´
/// ´test:crate:journal-open-failure-refuses-the-build´
#[test]
fn journal_open_failure_refuses_the_build() {
    let dir = test_dir();
    let config = test_persistence_config(Some(dir.path()));

    // A directory standing where the journal file belongs: the parent exists,
    // so setup succeeds and only the append-open can fail.
    let journal_path = config
        .persistence
        .as_ref()
        .expect("this config carries persistence")
        .journal_path();
    fs::create_dir_all(&journal_path).expect("the obstruction is created");

    let result = crate::Assayer::build(config);
    assert!(
        matches!(result, Err(crate::BuildError::PersistenceSetupFailed(_))),
        "an unopenable journal should refuse the build: {:?}",
        result.map(|_| "built")
    );
}

/// After a restart over a checkpoint-truncated journal, the writer numbers
/// above the checkpoint's mark rather than from one. The mark is the highest
/// sequence the model has already absorbed, and the replay filter keeps only
/// entries above it, so a writer that restarted below it would journal a whole
/// run's labels into the range replay discards — acknowledging them as durable
/// and then reading, parsing and passing over them on the next start.
///
/// ´claim:persistence:the-writer-numbers-above-the-checkpoints-mark-after-a-truncated-journal´
/// ´test:crate:construction-numbers-above-the-checkpoint-mark´
#[test]
fn construction_numbers_above_the_checkpoint_mark() {
    let dir = test_dir();
    let config = test_persistence_config(Some(dir.path()));
    let persistence = config.persistence.as_ref().expect("this config carries persistence").clone();
    fs::create_dir_all(&persistence.journal_dir).expect("journal dir");
    fs::create_dir_all(&persistence.checkpoint_dir).expect("checkpoint dir");

    // A checkpoint that absorbed five hundred labels and truncated the journal
    // to nothing, which is what the truncation does today.
    let mut working = test_working_copy();
    working.last_processed_label_seq = 500;
    checkpoint::write_checkpoint(&persistence.checkpoint_path(), &test_payload(&working)).expect("write failed");
    assert!(!persistence.journal_path().exists(), "the journal is truncated away");

    let assayer = crate::Assayer::build(config).expect("assayer builds");
    let next = assayer
        .journal
        .as_ref()
        .expect("persistence configured means a writer")
        .lock()
        .expect("uncontended")
        .next_seq();
    drop(assayer);

    assert!(
        next > 500,
        "the writer restarted at {next}, at or below the checkpoint's mark of 500"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Checkpoint Extension Tests
// ═══════════════════════════════════════════════════════════════════════════════

use crate::identity::{CellOutcomeState, CompetitiveCellId, IdentityGraphSnapshot};
use crate::ledger::entry::LedgerEntry;
use crate::ledger::sentinel_ledger::SentinelLedger;
use crate::persistence::checkpoint::{IdentityDimensionPayload, SentinelLedgerPayload};
use crate::types::{DimensionId, LedgerKey};

/// Creates a test sentinel ledger with a root + `n_extra` entries.
fn test_sentinel_ledger(n_extra: usize) -> SentinelLedger {
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();
    for i in 0..n_extra {
        let key = LedgerKey::new(u128::try_from((i + 1) * 0x100).unwrap(), 8);
        let mut entry = LedgerEntry::new_neutral();
        #[allow(clippy::cast_precision_loss)]
        {
            entry.ewma_bad_rate = 0.01 * (i + 1) as f64;
        }
        entry.total_assessments = (i + 1) as u64 * 10;
        ledger.insert(key, entry);
    }
    ledger
}

/// Creates identity dimension payloads with the given competitive cells.
fn test_identity_payload(cells: &[CompetitiveCellId]) -> IdentityDimensionPayload {
    let cell_outcomes: Vec<(CompetitiveCellId, CellOutcomeState)> =
        cells.iter().map(|&c| (c, CellOutcomeState::new_neutral())).collect();

    IdentityDimensionPayload {
        graph_snapshot: IdentityGraphSnapshot::default(),
        competitive_cells: cells.to_vec(),
        cell_outcome_state: cell_outcomes,
    }
}

/// Each Sentinel's outcome ledger crosses the checkpoint entire — every entry
/// including the root, under its own Sentinel, with the stored rates intact to
/// the last bit. The ledger is a long-memory structure built from many
/// assessments; rebuilding it from scratch after a restart would take as long
/// as it took to learn.
///
/// ´claim:persistence:each-sentinels-ledger-crosses-the-checkpoint-entire-with-its-stored-rates´
/// ´test:crate:checkpoint-with-ledger´
#[test]
fn checkpoint_with_ledger() {
    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let working = test_working_copy();

    // Build ledger state: 2 sentinels, 10 entries each.
    let ledger_state = vec![
        (
            SentinelId(1),
            SentinelLedgerPayload {
                ledger: test_sentinel_ledger(9),
            },
        ),
        (
            SentinelId(2),
            SentinelLedgerPayload {
                ledger: test_sentinel_ledger(9),
            },
        ),
    ];

    let payload = working.to_checkpoint_payload(PersistentTimestamp::now(), ledger_state, Vec::new());

    checkpoint::write_checkpoint(&path, &payload).expect("write failed");
    let restored = checkpoint::read_checkpoint(&path).expect("read failed");

    // Verify ledger state round-tripped.
    assert_eq!(restored.ledger_state.len(), 2);

    for (id, lp) in &restored.ledger_state {
        assert!(*id == SentinelId(1) || *id == SentinelId(2), "unexpected sentinel ID {id:?}");
        assert_eq!(lp.ledger.entry_count(), 10, "root + 9 extra entries");
        assert!(lp.ledger.has_root());

        // Verify one specific entry's EWMA survived.
        let key = LedgerKey::new(0x100, 8);
        let entry = lp.ledger.get(&key).expect("entry at 0x100 should exist");
        assert!(
            (entry.ewma_bad_rate - 0.01).abs() < 1e-15,
            "EWMA bad_rate should be 0.01, got {}",
            entry.ewma_bad_rate,
        );
    }
}

/// A dimension's identity state crosses the checkpoint under its own dimension
/// with its competitive cells and one outcome record per cell. The cells and
/// their outcomes are stored as parallel structures and must arrive as a
/// matched set: a restored dimension holding cells it has no outcome state for
/// would be a dimension that cannot say anything about the entities it routes.
///
/// ´claim:persistence:a-dimensions-competitive-cells-and-their-outcome-records-restore-as-a-matched-set´
/// ´test:crate:checkpoint-with-identity´
#[test]
fn checkpoint_with_identity() {
    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let working = test_working_copy();

    let cells: Vec<CompetitiveCellId> = (0..10)
        .map(|i| CompetitiveCellId::new(u128::try_from(i * 256).unwrap(), 8))
        .collect();

    let identity_state = vec![(DimensionId(1), test_identity_payload(&cells))];

    let payload = working.to_checkpoint_payload(PersistentTimestamp::now(), Vec::new(), identity_state);

    checkpoint::write_checkpoint(&path, &payload).expect("write failed");
    let restored = checkpoint::read_checkpoint(&path).expect("read failed");

    assert_eq!(restored.identity_state.len(), 1);
    let (dim_id, id_payload) = &restored.identity_state[0];
    assert_eq!(*dim_id, DimensionId(1));
    assert_eq!(id_payload.competitive_cells.len(), 10);
    assert_eq!(id_payload.cell_outcome_state.len(), 10);
}

/// Cell outcome rates survive the checkpoint value for value and cell by cell:
/// five cells given five distinct rates come back with each rate still on the
/// cell it belonged to, matching to the last representable digit. Counting the
/// cells is not enough — what makes a cell useful is the rate attached to it,
/// and a permutation would be invisible to any count.
///
/// ´claim:persistence:cell-outcome-rates-survive-value-for-value-on-the-cells-they-belong-to´
/// ´test:crate:checkpoint-with-cell-outcome-state´
#[test]
fn checkpoint_with_cell_outcome_state() {
    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let working = test_working_copy();

    let cells: Vec<CompetitiveCellId> = (0..5)
        .map(|i| CompetitiveCellId::new(u128::try_from(i * 256).unwrap(), 8))
        .collect();

    // Build cell outcomes with known EWMAs.
    let cell_outcomes: Vec<(CompetitiveCellId, CellOutcomeState)> = cells
        .iter()
        .enumerate()
        .map(|(i, &cell)| {
            let mut state = CellOutcomeState::new_neutral();
            #[allow(clippy::cast_precision_loss)]
            {
                state.adverse_rate_ewma = 0.1 * (i + 1) as f64;
            }
            (cell, state)
        })
        .collect();

    let identity_state = vec![(
        DimensionId(1),
        IdentityDimensionPayload {
            graph_snapshot: IdentityGraphSnapshot::default(),
            competitive_cells: cells,
            cell_outcome_state: cell_outcomes,
        },
    )];

    let payload = working.to_checkpoint_payload(PersistentTimestamp::now(), Vec::new(), identity_state);

    checkpoint::write_checkpoint(&path, &payload).expect("write failed");
    let restored = checkpoint::read_checkpoint(&path).expect("read failed");

    let (_, id_payload) = &restored.identity_state[0];
    for (i, (_, state)) in id_payload.cell_outcome_state.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let expected = 0.1 * (i + 1) as f64;
        assert!(
            (state.adverse_rate_ewma - expected).abs() < 1e-15,
            "cell {i}: EWMA should be {expected}, got {}",
            state.adverse_rate_ewma,
        );
    }
}

/// The refusal names both versions: a checkpoint claiming the very first
/// format is rejected with the version it declared and the version this build
/// expects. An operator reading the log needs to know which direction the
/// mismatch runs — an old file left by a previous deployment, or a newer one
/// written by a binary that has since been rolled back.
///
/// (´claim:persistence:a-checkpoint-of-another-format-version-is-refused-on-the-version´)
/// ´test:crate:checkpoint-format-version-bump´
#[test]
fn checkpoint_format_version_bump() {
    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let working = test_working_copy();
    let payload = test_payload(&working);
    checkpoint::write_checkpoint(&path, &payload).expect("write failed");

    // Overwrite version field with 1 (Layer 1 version) to simulate old checkpoint.
    let mut data = fs::read(&path).unwrap();
    let old_version: u32 = 1;
    data[4..8].copy_from_slice(&old_version.to_le_bytes());
    fs::write(&path, &data).unwrap();

    match checkpoint::read_checkpoint(&path) {
        Err(checkpoint::CheckpointError::VersionMismatch { found, expected }) => {
            assert_eq!(found, 1);
            assert_eq!(expected, 20, "current version should be 20");
        }
        other => panic!("expected VersionMismatch, got {other:?}"),
    }
}

/// A checkpoint of the immediately preceding generation is refused on the
/// version before its payload is read. Its payload carries the same fields as
/// this layout but was written by the previous codec, and the two encodings
/// agree byte for byte on integers below 128 — so a decoder handed those bytes
/// would not reliably fail on them. It would read some files and misread
/// others from the first divergence onward, which is worse than refusing every
/// one of them. The codec is not self-describing, so nothing in the payload
/// announces which encoding wrote it. The checkpoint format has no migration
/// path: refusal is the structural-compatibility rule
/// (´dec:durability:structural-compatibility´).
///
/// (´claim:persistence:a-checkpoint-of-another-format-version-is-refused-on-the-version´)
/// ´test:crate:checkpoint-prior-version-refused´
#[test]
fn checkpoint_prior_version_refused() {
    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let working = test_working_copy();
    let payload = test_payload(&working);
    checkpoint::write_checkpoint(&path, &payload).expect("write failed");

    // Rewrite the header's version field to the prior format version (19),
    // leaving the payload bytes as they are — the shape an un-upgraded
    // file presents is a version the reader must refuse before it ever
    // looks at the payload.
    let mut data = fs::read(&path).unwrap();
    let prior_version: u32 = 19;
    data[4..8].copy_from_slice(&prior_version.to_le_bytes());
    fs::write(&path, &data).unwrap();

    match checkpoint::read_checkpoint(&path) {
        Err(checkpoint::CheckpointError::VersionMismatch { found, expected }) => {
            assert_eq!(found, 19);
            assert_eq!(expected, 20);
        }
        other => panic!("expected VersionMismatch, got {other:?}"),
    }
}

/// Stored averages are aged once, in bulk, at the moment of restore: a ledger
/// rate saved a day ago comes back smaller than it was but still positive, and
/// its own last-updated stamp is moved forward to now. Both halves matter
/// together — the decay accounts for the downtime, and restamping the clock is
/// what stops the same gap being charged again the next time that entry is
/// touched.
///
/// ´claim:persistence:stored-averages-are-aged-once-on-restore-and-their-clocks-restamped´
/// ´test:crate:checkpoint-ledger-decay-on-restore´
#[test]
fn checkpoint_ledger_decay_on_restore() {
    // Ledger entries have `last_updated` timestamps. After checkpoint restore with
    // a 24h-old checkpoint, bulk restore-decay (´dec:durability:decay-once´) decays all stored
    // EWMAs and resets `last_updated` to now.
    let dir = test_dir();
    let cp_path = dir.path().join("checkpoint.bin");
    let jl_path = dir.path().join("journal.bin");

    let working = test_working_copy();

    // Create a ledger entry with known EWMA and old timestamp.
    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();
    let key = LedgerKey::new(0x100, 8);
    let mut entry = LedgerEntry::new_neutral();
    entry.ewma_bad_rate = 0.5;
    // Set last_updated to 24 hours ago.
    let now_secs = PersistentTimestamp::now().seconds;
    entry.last_updated = PersistentTimestamp::new(now_secs - 24 * 3600, 0);
    ledger.insert(key, entry);

    let ledger_state = vec![(SentinelId(1), SentinelLedgerPayload { ledger })];

    let past = PersistentTimestamp::new(now_secs - 24 * 3600, 0);
    let payload = working.to_checkpoint_payload(past, ledger_state, Vec::new());
    checkpoint::write_checkpoint(&cp_path, &payload).expect("write failed");
    fs::write(&jl_path, b"").unwrap();

    let config = test_persistence_config(None);

    let result = recovery::attempt_restore(&cp_path, &jl_path, &config, &[], &[], PersistentTimestamp::now())
        .expect("the fixture checkpoint is not refused on its numbers")
        .unwrap();

    // Bulk restore-decay (´dec:durability:decay-once´) applies γ_t_ledger^Δt to all
    // ledger EWMAs on restore and resets last_updated to now.
    let (_, lp) = &result.ledger_state[0];
    let restored_entry = lp.ledger.get(&key).expect("entry should exist");
    assert!(
        restored_entry.ewma_bad_rate < 0.5,
        "EWMA should have been decayed from 0.5: got {}",
        restored_entry.ewma_bad_rate,
    );
    assert!(
        restored_entry.ewma_bad_rate > 0.0,
        "EWMA should still be positive after decay",
    );
    // last_updated should be reset to approximately now, not the old timestamp.
    assert!(
        restored_entry.last_updated.seconds > now_secs - 10,
        "last_updated should be reset to now after bulk decay",
    );
}

/// Downtime is measured on the clock the engine was given, not on the wall.
/// A checkpoint captured and restored under an injected clock that advanced
/// exactly a day is aged by exactly a day's worth of decay, to the last place
/// the arithmetic holds.
///
/// The two ends of that subtraction have to come from one time domain. The
/// checkpoint's timestamp is written from the engine's clock, so a restore
/// that read the wall clock was subtracting a virtual instant from a real one:
/// the gap it computed was the distance between the two domains rather than
/// the downtime, and at the scale that distance reaches it is the elapsed
/// ceiling that answers, ageing every stored average by a year of absence that
/// never happened. A deterministic replay would then depend on the day it was
/// run.
///
/// ´claim:persistence:restore-measures-downtime-on-the-injected-clock-rather-than-the-wall´
/// ´test:crate:restore-measures-downtime-on-the-injected-clock´
#[test]
fn restore_measures_downtime_on_the_injected_clock() {
    use crate::numerics::decay_factor;
    use crate::testing::{Clock, VirtualClock};

    let dir = test_dir();
    let cp_path = dir.path().join("checkpoint.bin");
    let jl_path = dir.path().join("journal.bin");

    let clock = VirtualClock::epoch();
    let captured = clock.now();

    let working = test_working_copy();

    let mut ledger = SentinelLedger::new();
    ledger.ensure_root();
    let key = LedgerKey::new(0x100, 8);
    let mut entry = LedgerEntry::new_neutral();
    entry.ewma_bad_rate = 0.5;
    entry.last_updated = captured;
    ledger.insert(key, entry);

    let ledger_state = vec![(SentinelId(1), SentinelLedgerPayload { ledger })];
    let payload = working.to_checkpoint_payload(captured, ledger_state, Vec::new());
    checkpoint::write_checkpoint(&cp_path, &payload).expect("write failed");
    fs::write(&jl_path, b"").unwrap();

    // The whole of the downtime, and the only downtime there is.
    clock.advance(Duration::from_secs(24 * 3600));

    let config = test_persistence_config(None);
    let result = recovery::attempt_restore(&cp_path, &jl_path, &config, &[], &[], clock.now())
        .expect("the fixture checkpoint is not refused on its numbers")
        .unwrap();

    let expected = 0.5 * decay_factor(config.temporal.gamma_t_ledger, 24.0);
    let (_, lp) = &result.ledger_state[0];
    let restored_entry = lp.ledger.get(&key).expect("entry should exist");
    assert!(
        (restored_entry.ewma_bad_rate - expected).abs() < 1e-12,
        "a day on the injected clock ages the rate by a day: expected {expected}, got {}",
        restored_entry.ewma_bad_rate,
    );
    assert_eq!(
        restored_entry.last_updated,
        clock.now(),
        "the restamped clock is the injected one too",
    );
}

/// A journal corrupt before its tail fails the restore instead of yielding an
/// empty replay set, while the two failures the protocol absorbs still restore
/// cleanly: a record torn at the tail costs only itself, and a journal of
/// another format version is discarded by design.
///
/// The entries past the checkpoint's mark are the only copy of every label
/// acknowledged since the capture. A corruption before the tail makes every
/// record after it unfindable, so proceeding with no replay would discard
/// evidence the host was told was durable and report a successful start over
/// the gap. The host has real actions at that moment — an earlier checkpoint,
/// a repaired journal, a deliberate cold start — and construction has to fail
/// for it to take one knowingly. The torn tail is the opposite case and stays
/// absorbed: it is the single label the append protocol allows to be lost, and
/// refusing the build over it would turn every crash into an outage. So is the
/// version mismatch, which is how an upgraded binary meets the previous
/// deployment's journal.
///
/// ´claim:persistence:a-journal-corrupt-before-its-tail-fails-the-restore-while-the-absorbed-failures-still-restore´
/// ´test:crate:restore-refuses-a-journal-corrupt-before-its-tail´
#[test]
fn restore_refuses_a_journal_corrupt_before_its_tail() {
    let dir = test_dir();
    let cp_path = dir.path().join("checkpoint.bin");
    let jl_path = dir.path().join("journal.bin");

    let working = test_working_copy();
    let payload = working.to_checkpoint_payload(PersistentTimestamp::now(), Vec::new(), Vec::new());
    checkpoint::write_checkpoint(&cp_path, &payload).expect("write failed");

    let config = test_persistence_config(None);

    // A well-formed journal of three records, kept as the fixture every arm
    // below starts from.
    {
        let mut file = journal::open_journal_for_append(&jl_path).unwrap();
        for i in 1..=3 {
            journal::append_journal_entry(&mut file, &test_journal_entry(i)).unwrap();
        }
    }
    let intact = fs::read(&jl_path).unwrap();

    let result = recovery::attempt_restore(&cp_path, &jl_path, &config, &[], &[], PersistentTimestamp::now())
        .expect("an intact journal restores")
        .expect("the checkpoint is there to restore");
    assert_eq!(result.replay_entries.len(), 3, "the intact fixture replays all three");

    // A corruption before the tail: a well-formed length prefix over four
    // bytes that cannot deserialise, spliced in ahead of the three complete
    // records, so the reader meets it with the whole file still to come and
    // cannot mistake it for a torn tail.
    let mut corrupt = intact[..8].to_vec();
    corrupt.extend_from_slice(&4_u32.to_le_bytes());
    corrupt.extend_from_slice(&[0xFF_u8; 4]);
    corrupt.extend_from_slice(&intact[8..]);
    fs::write(&jl_path, &corrupt).unwrap();

    match recovery::attempt_restore(&cp_path, &jl_path, &config, &[], &[], PersistentTimestamp::now()) {
        Err(crate::error::BuildError::JournalUnreadable { reason }) => {
            assert!(
                reason.contains("corrupt"),
                "the refusal reports what the reader refused on: {reason}"
            );
        }
        Err(e) => panic!("expected JournalUnreadable, got {e}"),
        Ok(_) => panic!("a journal corrupt before its tail must not restore with an empty replay set"),
    }

    // A record torn at the tail is still absorbed, and the complete records
    // before it still replay.
    let mut torn = intact.clone();
    let len = torn.len();
    torn.truncate(len - 5);
    fs::write(&jl_path, &torn).unwrap();

    let result = recovery::attempt_restore(&cp_path, &jl_path, &config, &[], &[], PersistentTimestamp::now())
        .expect("a torn tail is absorbed, not refused")
        .expect("the checkpoint is there to restore");
    assert_eq!(
        result.replay_entries.len(),
        2,
        "the two complete records survive the torn tail"
    );

    // A journal of another format version is discarded by design, and the
    // build proceeds with nothing to replay.
    let mut other_version = Vec::new();
    other_version.extend_from_slice(b"ASAJ");
    other_version.extend_from_slice(&999_u32.to_le_bytes());
    fs::write(&jl_path, &other_version).unwrap();

    let result = recovery::attempt_restore(&cp_path, &jl_path, &config, &[], &[], PersistentTimestamp::now())
        .expect("a journal of another format version is discarded, not refused")
        .expect("the checkpoint is there to restore");
    assert!(
        result.replay_entries.is_empty(),
        "the discarded journal contributes nothing to replay"
    );
}

/// Identity cell rates are aged by the same restore-time pass, under their own
/// forgetting rate rather than the ledger's: a day-old adverse rate returns
/// reduced, still positive, and restamped to now. The bulk decay reaches every
/// family of stored averages the checkpoint carries, not only the core models.
///
/// (´claim:persistence:stored-averages-are-aged-once-on-restore-and-their-clocks-restamped´)
/// ´test:crate:checkpoint-identity-decay-on-restore´
#[test]
fn checkpoint_identity_decay_on_restore() {
    // Cell outcome state uses the same bulk restore-decay pattern as ledger.
    // After restore, EWMAs are decayed and `last_updated` is reset to now.
    let dir = test_dir();
    let cp_path = dir.path().join("checkpoint.bin");
    let jl_path = dir.path().join("journal.bin");

    let working = test_working_copy();

    let cell = CompetitiveCellId::new(0x100, 8);
    let mut state = CellOutcomeState::new_neutral();
    state.adverse_rate_ewma = 0.8;
    let now_secs = PersistentTimestamp::now().seconds;
    state.last_updated = PersistentTimestamp::new(now_secs - 24 * 3600, 0);

    let identity_state = vec![(
        DimensionId(1),
        IdentityDimensionPayload {
            graph_snapshot: IdentityGraphSnapshot::default(),
            competitive_cells: vec![cell],
            cell_outcome_state: vec![(cell, state)],
        },
    )];

    let past = PersistentTimestamp::new(now_secs - 24 * 3600, 0);
    let payload = working.to_checkpoint_payload(past, Vec::new(), identity_state);
    checkpoint::write_checkpoint(&cp_path, &payload).expect("write failed");
    fs::write(&jl_path, b"").unwrap();

    let config = test_persistence_config(None);

    let result = recovery::attempt_restore(&cp_path, &jl_path, &config, &[], &[], PersistentTimestamp::now())
        .expect("the fixture checkpoint is not refused on its numbers")
        .unwrap();

    let (_, id_payload) = &result.identity_state[0];
    let (_, restored_state) = &id_payload.cell_outcome_state[0];
    // Bulk restore-decay (´dec:durability:decay-once´) applies γ_t_identity^Δt to
    // identity cell EWMAs on restore and resets last_updated to now.
    assert!(
        restored_state.adverse_rate_ewma < 0.8,
        "EWMA should have been decayed from 0.8: got {}",
        restored_state.adverse_rate_ewma,
    );
    assert!(
        restored_state.adverse_rate_ewma > 0.0,
        "EWMA should still be positive after decay",
    );
    assert!(
        restored_state.last_updated.seconds > now_secs - 10,
        "last_updated should be reset to now after bulk decay",
    );
}

/// The parts of the state travel together rather than merely each travelling:
/// a checkpoint carrying core scalars, a registered outcome axis with its
/// spatial flag, two Sentinel ledgers of different sizes, a dimension's
/// identity state and the dimension map restores all of them at once, and the
/// working copy rebuilt from it agrees on the total feature width. It is the
/// agreement between the pieces that makes the state usable — a dimension map
/// that no longer matches the models it indexes would restore cleanly and then
/// be wrong.
///
/// ´claim:persistence:the-parts-of-the-state-restore-together-and-still-agree-with-one-another´
/// ´test:crate:checkpoint-complete-round-trip´
#[test]
fn checkpoint_complete_round_trip() {
    use indexmap::IndexMap;

    use crate::feature::dimension_map::DimensionMap;
    use crate::snapshot::working::WorkingAxisModel;

    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    // Build a working copy with 1 outcome model.
    let mut working = test_working_copy();
    working.kappa_v = 2.5;
    working.p_positive_global = 0.3;
    working.last_processed_label_seq = 42;
    working.outcome_models.insert(
        OutcomeAxisId(1),
        WorkingAxisModel {
            name: "axis_1".to_owned(),
            description: "axis one description".to_owned(),
            model: crate::model::bayesian::BayesianLinearModel::new(10, 0.1, 1000),
            kappa_a: 1.5,
            gamma: 0.9998,
            spatial: true,
            eligibility: crate::types::OutcomeEligibility::default(),
        },
    );
    working.dimension_map = DimensionMap::rebuild_no_interactions(
        0,
        &IndexMap::from([(SentinelId(1), ()), (SentinelId(2), ())]),
        &[DimensionId(1)],
        &IndexMap::new(),
        &[],
    );

    // Build ledger state.
    let ledger_state = vec![
        (
            SentinelId(1),
            SentinelLedgerPayload {
                ledger: test_sentinel_ledger(5),
            },
        ),
        (
            SentinelId(2),
            SentinelLedgerPayload {
                ledger: test_sentinel_ledger(3),
            },
        ),
    ];

    // Build identity state.
    let cells: Vec<CompetitiveCellId> = (0..3)
        .map(|i| CompetitiveCellId::new(u128::try_from(i * 256).unwrap(), 8))
        .collect();
    let identity_state = vec![(DimensionId(1), test_identity_payload(&cells))];

    let payload = working.to_checkpoint_payload(PersistentTimestamp::now(), ledger_state, identity_state);

    checkpoint::write_checkpoint(&path, &payload).expect("write failed");
    let restored = checkpoint::read_checkpoint(&path).expect("read failed");

    // Verify all components.
    assert_eq!(restored.last_processed_label_seq, 42);
    assert_eq!(restored.kappa_v, 2.5);
    assert_eq!(restored.p_positive_global, 0.3);
    assert_eq!(restored.outcome_models.len(), 1);
    assert!(restored.outcome_models[0].1.spatial);
    assert_eq!(restored.ledger_state.len(), 2);
    assert_eq!(restored.identity_state.len(), 1);
    assert_eq!(restored.dimension_map.sentinel_slots.len(), 2);
    assert_eq!(restored.dimension_map.id_dim_ranges.len(), 1);
    assert!(restored.dimension_map.id_cross_dim_range.is_some());

    // Verify ledger entry counts.
    let (_, lp1) = &restored.ledger_state[0];
    assert_eq!(lp1.ledger.entry_count(), 6); // root + 5
    let (_, lp2) = &restored.ledger_state[1];
    assert_eq!(lp2.ledger.entry_count(), 4); // root + 3

    // Verify identity cells.
    let (_, id_payload) = &restored.identity_state[0];
    assert_eq!(id_payload.competitive_cells.len(), 3);

    // DimensionMap: reconstruct working copy and verify dual tracking.
    let wc2 = crate::snapshot::working::WorkingCopy::from_checkpoint_payload(&restored, 100, 0.1)
        .expect("the fixture checkpoint carries definite precision matrices");
    assert_eq!(wc2.dimension_map.p, working.dimension_map.p);
    assert!(wc2.outcome_models[&OutcomeAxisId(1)].spatial);
}

/// An axis registered away from every default comes back from a restart as it
/// was registered: the name, the description and the all-labels eligibility
/// mode all cross the checkpoint and are read back off the restored working
/// copy, not substituted. The eligibility mode decides which labels the axis
/// model trains on (´def:axis:training-target´), so an axis silently reverted
/// to the eligible-only default would train on a differently-selected
/// population from the restart onward with no surface reporting it.
///
/// ´claim:persistence:an-axis-returns-from-a-restart-with-its-name-description-and-eligibility´
/// ´test:crate:checkpoint-axis-fields-round-trip´
#[test]
fn checkpoint_axis_fields_round_trip() {
    use crate::snapshot::working::WorkingAxisModel;
    use crate::types::OutcomeEligibility;

    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let mut working = test_working_copy();
    working.outcome_models.insert(
        OutcomeAxisId(7),
        WorkingAxisModel {
            name: "chargeback".to_owned(),
            description: "probability the payment is later reversed".to_owned(),
            model: crate::model::bayesian::BayesianLinearModel::new(16, 0.1, 1000),
            kappa_a: 2.0,
            gamma: 0.9997,
            spatial: false,
            // Away from the default: the default is EligibleOnly, so a
            // restore that substitutes defaults is visible here.
            eligibility: OutcomeEligibility::AllLabels,
        },
    );

    let payload = test_payload(&working);
    checkpoint::write_checkpoint(&path, &payload).expect("write failed");
    let restored_payload = checkpoint::read_checkpoint(&path).expect("read failed");

    // Checkpoint out: the payload itself carries the three fields.
    let (_, axis_state) = &restored_payload.outcome_models[0];
    assert_eq!(axis_state.name, "chargeback");
    assert_eq!(axis_state.description, "probability the payment is later reversed");
    assert_eq!(axis_state.eligibility, OutcomeEligibility::AllLabels);

    // Restore in: the rebuilt working copy reads them off the payload.
    let restored = crate::snapshot::working::WorkingCopy::from_checkpoint_payload(&restored_payload, 100, 0.1)
        .expect("the fixture checkpoint carries definite precision matrices");
    let wam = &restored.outcome_models[&OutcomeAxisId(7)];
    assert_eq!(wam.name, "chargeback");
    assert_eq!(wam.description, "probability the payment is later reversed");
    assert_eq!(wam.eligibility, OutcomeEligibility::AllLabels);
}

/// Drift evidence survives a restart: accumulators fed to distinct values on
/// three models cross the checkpoint and come back on the rebuilt working
/// copy value for value — both CUSUM sides, both smoothed diagnostics and the
/// step counter, each still on the model that accumulated it. A restart is
/// not among the four discard triggers (´tab:monitoring:drift-resets´), so a
/// restore that handed every drifting model a fresh start would be a reset
/// the table does not admit, firing on every deployment's ordinary cadence.
///
/// ´claim:persistence:drift-evidence-crosses-the-checkpoint-instead-of-resetting´
/// ´test:crate:checkpoint-drift-state-round-trip´
#[test]
fn checkpoint_drift_state_round_trip() {
    use crate::health::DriftState;
    use crate::types::ModelId;

    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let mut working = test_working_copy();
    for (i, model_id) in [ModelId::Operational, ModelId::Sister, ModelId::OutcomeAxis(OutcomeAxisId(3))]
        .into_iter()
        .enumerate()
    {
        let scale = f64::from(u32::try_from(i).unwrap()) + 1.0;
        working.drift_state.insert(
            model_id,
            DriftState {
                s_plus: 0.25 * scale,
                s_minus: 0.125 * scale,
                mean_abs_residual: 0.5 * scale,
                residual_sign_ewma: 0.25f64.mul_add(-scale, 0.75),
                steps_since_reset: 100 + u64::try_from(i).unwrap(),
            },
        );
    }

    let payload = test_payload(&working);
    checkpoint::write_checkpoint(&path, &payload).expect("write failed");
    let restored_payload = checkpoint::read_checkpoint(&path).expect("read failed");

    let restored = crate::snapshot::working::WorkingCopy::from_checkpoint_payload(&restored_payload, 100, 0.1)
        .expect("the fixture checkpoint carries definite precision matrices");
    assert_eq!(restored.drift_state.len(), working.drift_state.len());
    for (model_id, drift) in &working.drift_state {
        let back = restored
            .drift_state
            .get(model_id)
            .unwrap_or_else(|| panic!("drift state for {model_id:?} did not cross the checkpoint"));
        assert_eq!(back.s_plus, drift.s_plus);
        assert_eq!(back.s_minus, drift.s_minus);
        assert_eq!(back.mean_abs_residual, drift.mean_abs_residual);
        assert_eq!(back.residual_sign_ewma, drift.residual_sign_ewma);
        assert_eq!(back.steps_since_reset, drift.steps_since_reset);
    }
}

/// The standardisation statistics cross the checkpoint as values, not as
/// shapes: means, variances and the class vector all come back on the rebuilt
/// working copy exactly as they stood at capture, with the class assignments
/// still on the positions they described. Every feature every model reads is
/// scaled by these vectors, so a restore that reset them to the neutral pair
/// would mis-scale the whole feature vector for the standardisation
/// half-life after every restart (´req:standardisation:timing´).
///
/// ´claim:persistence:the-standardisation-statistics-cross-the-checkpoint-as-values´
/// ´test:crate:checkpoint-standardisation-round-trip´
#[test]
fn checkpoint_standardisation_round_trip() {
    use crate::feature::standardisation::FeatureClass;

    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let mut working = test_working_copy();
    let p = working.dimension_map.p;
    for j in 0..p {
        let x = f64::from(u32::try_from(j).unwrap());
        working.feature_means[j] = 0.1f64.mul_add(x, 0.05);
        working.feature_variances[j] = 0.01f64.mul_add(x, 0.5);
    }
    working.feature_classes[0] = FeatureClass::Bias;
    working.feature_classes[p - 1] = FeatureClass::Cusum;

    let payload = test_payload(&working);
    checkpoint::write_checkpoint(&path, &payload).expect("write failed");
    let restored_payload = checkpoint::read_checkpoint(&path).expect("read failed");

    // Checkpoint out: the payload carries the three vectors.
    assert_eq!(restored_payload.feature_means, working.feature_means);
    assert_eq!(restored_payload.feature_variances, working.feature_variances);
    assert_eq!(restored_payload.feature_classes, working.feature_classes);

    // Restore in: the rebuilt working copy reads them off the payload.
    let restored = crate::snapshot::working::WorkingCopy::from_checkpoint_payload(&restored_payload, 100, 0.1)
        .expect("the fixture checkpoint carries definite precision matrices");
    assert_eq!(restored.feature_means, working.feature_means);
    assert_eq!(restored.feature_variances, working.feature_variances);
    assert_eq!(restored.feature_classes, working.feature_classes);
}

/// A checkpoint taken part way along the cold ramp comes back as the same ramp:
/// the base it was mixing against, the sample it had accepted and the count
/// that positions it all cross the file, and the restored instance publishes
/// exactly the moments the running one published and reports exactly the same
/// phase and count. A checkpoint that carried the published pair and nothing
/// behind it would leave a restore only two answers, both wrong — restart the
/// ramp, and pay the whole cold start again at every restart; or treat a
/// partial transition as a finished one, and publish a mixture as though it
/// were the empirical moments. Carrying it whole is what makes a resumed ramp
/// publish what an uninterrupted one would have published at the same count,
/// rather than one run's sample against another run's assumptions.
///
/// ´claim:persistence:a-mid-ramp-checkpoint-resumes-the-same-count-base-and-sample´
/// ´test:crate:checkpoint-mid-ramp-resumes-where-it-stopped´
#[test]
fn checkpoint_mid_ramp_resumes_where_it_stopped() {
    use crate::feature::standardisation::StandardisationPhase;

    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let mut working = test_working_copy();
    let width = working.feature_means.len();
    let generation = working.layout_generation;
    let v_floor = 1e-4;

    // Walk part way along the ramp on a sample that is not constant, so the
    // accepted moments are something a restore could plausibly get wrong.
    for i in 0..7 {
        let x = f64::from(i);
        let observation: Vec<f64> = (0..width)
            .map(|j| f64::from(u32::try_from(j).unwrap()).mul_add(0.25, x))
            .collect();
        working
            .accept_cold_observation(&observation, generation, v_floor)
            .expect("mid-ramp observation is accepted");
    }
    assert_eq!(working.cold_ramp.phase(), StandardisationPhase::Transitioning);
    assert_eq!(working.cold_ramp.accepted(), 7);

    let horizon = working.cold_ramp.horizon();
    let published_means = working.feature_means.clone();
    let published_variances = working.feature_variances.clone();

    let payload = test_payload(&working);
    checkpoint::write_checkpoint(&path, &payload).expect("write failed");
    let restored_payload = checkpoint::read_checkpoint(&path).expect("read failed");

    // Out: the parts travel, not just the pair they produced.
    assert_eq!(restored_payload.cold_ramp.accepted, 7);
    assert_eq!(restored_payload.cold_ramp.phase, StandardisationPhase::Transitioning);
    assert_eq!(restored_payload.cold_ramp.base_means, working.cold_ramp.base_means());
    assert_eq!(restored_payload.cold_ramp.base_variances, working.cold_ramp.base_variances());
    assert_eq!(restored_payload.cold_ramp.layout_generation, generation);

    // In: the ramp resumes at the count it stopped at, and publishing from it
    // reproduces what the running instance had published.
    let restored = crate::snapshot::working::WorkingCopy::from_checkpoint_payload(&restored_payload, horizon, 0.1)
        .expect("the fixture checkpoint carries definite precision matrices");
    assert_eq!(restored.cold_ramp.accepted(), 7);
    assert_eq!(restored.cold_ramp.phase(), StandardisationPhase::Transitioning);
    assert_eq!(restored.feature_means, published_means);
    assert_eq!(restored.feature_variances, published_variances);

    let (again_means, again_variances) = restored.cold_ramp.publish(&restored.feature_classes, v_floor);
    assert_eq!(again_means, published_means, "a resumed ramp republishes the same moments");
    assert_eq!(again_variances, published_variances);

    // And it goes on from there rather than starting again: the next accepted
    // observation is the eighth, not the first.
    let mut resumed = restored;
    let observation = vec![1.0; width];
    resumed
        .accept_cold_observation(&observation, generation, v_floor)
        .expect("the resumed ramp accepts");
    assert_eq!(resumed.cold_ramp.accepted(), 8);
}

/// A ramp that had reached its horizon comes back in service, and a ramp
/// restored against a horizon that has since been shortened completes on
/// arrival and publishes the empirical moments. The horizon is deliberately
/// not carried in the file: it is read from the configuration in force, so a
/// restore recomputes its own position rather than holding a second authority
/// that could disagree with the configuration it was restored under. A count
/// at or past the new horizon has no prior mass left to retire whatever it was
/// gathered against, and the one-time change in mixture weight is disclosed by
/// the restore's own coordinate-version change.
///
/// ´claim:persistence:a-restored-ramp-recomputes-its-position-against-the-horizon-in-force´
/// ´test:crate:checkpoint-ramp-recomputes-against-the-configured-horizon´
#[test]
fn checkpoint_ramp_recomputes_against_the_configured_horizon() {
    use crate::feature::standardisation::StandardisationPhase;

    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let mut working = test_working_copy();
    let width = working.feature_means.len();
    let generation = working.layout_generation;
    let v_floor = 1e-4;
    let horizon = working.cold_ramp.horizon();

    for i in 0..horizon {
        let x = f64::from(u32::try_from(i).unwrap());
        let observation: Vec<f64> = (0..width).map(|j| x + f64::from(u32::try_from(j).unwrap())).collect();
        working
            .accept_cold_observation(&observation, generation, v_floor)
            .expect("observation accepted");
    }
    assert_eq!(working.cold_ramp.phase(), StandardisationPhase::InService);

    let payload = test_payload(&working);
    checkpoint::write_checkpoint(&path, &payload).expect("write failed");
    let restored_payload = checkpoint::read_checkpoint(&path).expect("read failed");

    // Restored under the same horizon: in service, publishing the empirical
    // moments it reached.
    let same = crate::snapshot::working::WorkingCopy::from_checkpoint_payload(&restored_payload, horizon, 0.1)
        .expect("the fixture checkpoint carries definite precision matrices");
    assert_eq!(same.cold_ramp.phase(), StandardisationPhase::InService);
    assert_eq!(same.feature_means, working.feature_means);

    // Restored under a shortened horizon: the count is already past it, so the
    // ramp is complete on arrival rather than somehow more than complete.
    let shortened = crate::snapshot::working::WorkingCopy::from_checkpoint_payload(&restored_payload, horizon / 2, 0.1)
        .expect("the fixture checkpoint carries definite precision matrices");
    assert_eq!(shortened.cold_ramp.phase(), StandardisationPhase::InService);
    assert!(shortened.cold_ramp.is_complete());
    let (means, _) = shortened.cold_ramp.publish(&shortened.feature_classes, v_floor);
    let empirical = shortened.cold_ramp.accumulator().complete().expect("a sample was accepted");
    for (j, (published, observed)) in means.iter().zip(empirical.mean.iter()).enumerate().take(width).skip(1) {
        assert_eq!(
            published, observed,
            "a count past the horizon publishes the empirical moments at position {j}",
        );
    }

    // Restored under a lengthened horizon: the same sample now stands part way
    // along a longer ramp, and the phase says so rather than claiming service.
    let lengthened = crate::snapshot::working::WorkingCopy::from_checkpoint_payload(&restored_payload, horizon * 2, 0.1)
        .expect("the fixture checkpoint carries definite precision matrices");
    assert_eq!(lengthened.cold_ramp.phase(), StandardisationPhase::Transitioning);
    assert!((lengthened.cold_ramp.retired_share() - 0.5).abs() < 1e-12);
}

/// A restarted instance publishes the standardisation it had learned, through
/// the path a deployment takes: labels move the running statistics, an
/// acknowledged checkpoint captures them, and the instance rebuilt from that
/// file publishes the same vectors in its first snapshot — with the batch
/// initialisation left off, its premise now true rather than asserted. This
/// is the warm-start branch's ground made real: the fast path is switched off
/// because the restored statistics are empirical, not because a comment says
/// they are (´alg:standardisation:batch-initialisation´).
///
/// ´claim:persistence:a-restart-resumes-the-learned-standardisation-through-the-production-path´
/// ´test:crate:checkpoint-standardisation-warm-start-end-to-end´
#[test]
fn checkpoint_standardisation_warm_start_end_to_end() {
    use crate::owner::commands::CheckpointRequest as CpReq;

    let dir = test_dir();

    // Run: process labels so the running statistics leave neutral, then
    // checkpoint with acknowledgement and shut down.
    let (means_at_capture, variances_at_capture) = {
        let assayer = test_assayer_with_persistence(dir.path(), "warm-start");
        for i in 1..=20_u64 {
            assayer.label_tx().send(dummy_label(AssessmentId(i), Some(i))).unwrap();
        }
        let v = wait_for_version(assayer.shared(), 21, crate::testing::STATE_DEADLINE);
        assert_eq!(v, 21, "labels should be processed");

        let (ack_tx, ack_rx) = crossbeam_channel::bounded(1);
        assayer
            .command_tx()
            .send(crate::owner::commands::ModelOwnerCommand::Checkpoint(CpReq {
                completion: Some(ack_tx),
            }))
            .unwrap();
        ack_rx.recv_timeout(ACK_DEADLINE).unwrap().expect("checkpoint should succeed");

        let snapshot = assayer.shared().published.load();
        (snapshot.feature_means.clone(), snapshot.feature_variances.clone())
    };

    // The run's statistics must have left neutral for the assertion below
    // to distinguish a restore from a cold start.
    assert!(
        variances_at_capture.iter().any(|&v| v != 1.0),
        "labels should have moved the running variances off neutral"
    );

    // Restart from the same directory: the first published snapshot
    // carries the captured statistics, not the neutral pair.
    let rebuilt = test_assayer_with_persistence(dir.path(), "warm-start-2");
    let snapshot = rebuilt.shared().published.load();
    assert_eq!(snapshot.feature_means, means_at_capture);
    assert_eq!(snapshot.feature_variances, variances_at_capture);
}

/// A checkpoint carrying more Sentinels than the restoring instance knows
/// about restores anyway, and all of that ledger state is handed back for the
/// caller to reconcile. Sentinels are registered at runtime rather than fixed
/// at construction, so their population is not part of what makes a checkpoint
/// structurally compatible — only the feature geometry is. Discarding a
/// Sentinel's history because it had not re-registered yet would throw away
/// state that is about to become relevant again.
///
/// ´claim:persistence:runtime-registered-populations-are-not-part-of-the-structural-check´
/// ´test:crate:checkpoint-structural-mismatch-sentinel-count´
#[test]
fn checkpoint_structural_mismatch_sentinel_count() {
    // Currently, structural mismatch checking only validates anchor dimension.
    // Sentinel count mismatch is tolerated — the checkpoint's ledger state is
    // passed through to the caller for reconciliation.
    // This test documents the current behavior; a later layer may add stricter
    // guards, since the builder is where validation is meant to bite
    // (´dec:construction:eager-validation´).
    let dir = test_dir();
    let cp_path = dir.path().join("checkpoint.bin");
    let jl_path = dir.path().join("journal.bin");

    let working = test_working_copy();

    // Checkpoint with 3 sentinels.
    let ledger_state = vec![
        (
            SentinelId(1),
            SentinelLedgerPayload {
                ledger: test_sentinel_ledger(2),
            },
        ),
        (
            SentinelId(2),
            SentinelLedgerPayload {
                ledger: test_sentinel_ledger(2),
            },
        ),
        (
            SentinelId(3),
            SentinelLedgerPayload {
                ledger: test_sentinel_ledger(2),
            },
        ),
    ];
    let payload = working.to_checkpoint_payload(PersistentTimestamp::now(), ledger_state, Vec::new());
    checkpoint::write_checkpoint(&cp_path, &payload).expect("write failed");
    fs::write(&jl_path, b"").unwrap();

    // Config doesn't carry sentinel count — restore succeeds.
    let config = test_persistence_config(None);

    let result = recovery::attempt_restore(&cp_path, &jl_path, &config, &[], &[], PersistentTimestamp::now())
        .expect("the fixture checkpoint is not refused on its numbers");
    assert!(result.is_some(), "sentinel count mismatch is tolerated in Layer 2");
    assert_eq!(result.unwrap().ledger_state.len(), 3);
}

/// Identity dimensions are treated the same way: a checkpoint holding two of
/// them restores against a configuration that declares none, and both come back
/// for the caller to reconcile. Dimensions, like Sentinels, are registered
/// after construction, so their count says nothing about whether the feature
/// layout still matches.
///
/// (´claim:persistence:runtime-registered-populations-are-not-part-of-the-structural-check´)
/// ´test:crate:checkpoint-structural-mismatch-dimension-count´
#[test]
fn checkpoint_structural_mismatch_dimension_count() {
    // Same as above: dimension count mismatch is currently tolerated.
    // The checkpoint's identity state is passed through to the caller.
    let dir = test_dir();
    let cp_path = dir.path().join("checkpoint.bin");
    let jl_path = dir.path().join("journal.bin");

    let working = test_working_copy();

    let cells: Vec<CompetitiveCellId> = (0..3)
        .map(|i| CompetitiveCellId::new(u128::try_from(i * 256).unwrap(), 8))
        .collect();
    let identity_state = vec![
        (DimensionId(1), test_identity_payload(&cells)),
        (DimensionId(2), test_identity_payload(&cells)),
    ];
    let payload = working.to_checkpoint_payload(PersistentTimestamp::now(), Vec::new(), identity_state);
    checkpoint::write_checkpoint(&cp_path, &payload).expect("write failed");
    fs::write(&jl_path, b"").unwrap();

    let config = test_persistence_config(None);

    let result = recovery::attempt_restore(&cp_path, &jl_path, &config, &[], &[], PersistentTimestamp::now())
        .expect("the fixture checkpoint is not refused on its numbers");
    assert!(result.is_some(), "dimension count mismatch is tolerated in Layer 2");
    assert_eq!(result.unwrap().identity_state.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Assayer Integration Tests
// ═══════════════════════════════════════════════════════════════════════════════

use super::helpers::{AssayerAssessDeriveCompat, wait_for_version};
use crate::Assayer;
use crate::health::DegradationContext;
use crate::owner::commands::{LabelContext, PendingAssessment, SequencedLabel};
use crate::pending::PendingRiskBasis;

fn test_pending_assessment(id: AssessmentId) -> PendingAssessment {
    PendingAssessment {
        spatial_axis_ids: Vec::new(),
        id,
        timestamp: std::time::Instant::now(),
        persistent_timestamp: crate::types::PersistentTimestamp::now(),
        entity: EntityKey::new(id.0.to_le_bytes().to_vec()),
        sentinel_extractions: HashMap::new(),
        identity_coordinates: HashMap::new(),
        identity_active_cells: HashMap::new(),
        active_sentinels: Vec::new(),
        reporting_sentinels: Vec::new(),
        entity_base_features: HashMap::new(),
        entity_axis_features: HashMap::new(),
        signal_features: crate::pending::StoredFeatures::default(),
        risk_basis: PendingRiskBasis::default(),
        outcome_predictions: HashMap::new(),
        degradation: DegradationContext::default(),
        report_origin: None,
    }
}

/// Creates a test `Assayer` with persistence config.
fn test_assayer_with_persistence(dir: &std::path::Path, instance_id: &str) -> Assayer {
    let mut config = test_persistence_config(Some(dir));
    config.instance_id = instance_id.to_owned();
    Assayer::build(config).expect("assayer builds")
}

/// Creates a dummy `SequencedLabel` for testing.
fn dummy_label(assessment_id: AssessmentId, seq: Option<u64>) -> SequencedLabel {
    SequencedLabel {
        seq,
        label: test_label(assessment_id),
        context: LabelContext::Full(Box::new(test_pending_assessment(assessment_id))),
        arrived_at: None,
    }
}

/// A checkpoint taken through a running instance records the work it had
/// actually done: after ten labels have been processed and an explicit
/// checkpoint acknowledged, the file on disk names ten as its high-water mark
/// and carries the models at their configured widths. The acknowledgement is
/// what makes this meaningful — it is the model owner's statement that the
/// snapshot includes everything it had accepted, and it is the only thing
/// distinguishing a checkpoint from a request for one.
///
/// ´claim:persistence:an-acknowledged-checkpoint-records-every-label-the-owner-had-processed´
/// ´test:crate:checkpoint-round-trip-via-assayer´
#[test]
fn checkpoint_round_trip_via_assayer() {
    use crate::owner::commands::CheckpointRequest;

    let dir = test_dir();
    let cp_path = dir.path().join("checkpoint.bin");

    // Step 1: Build, send 10 labels, explicit checkpoint, shutdown.
    {
        let assayer = test_assayer_with_persistence(dir.path(), "cp-roundtrip");
        let timeout = crate::testing::STATE_DEADLINE;

        for i in 1..=10_u64 {
            assayer.label_tx().send(dummy_label(AssessmentId(i), Some(i))).unwrap();
        }
        let v = wait_for_version(assayer.shared(), 11, timeout);
        assert_eq!(v, 11, "10 labels should be processed");

        // Explicit checkpoint.
        let (ack_tx, ack_rx) = crossbeam_channel::bounded(1);
        assayer
            .command_tx()
            .send(crate::owner::commands::ModelOwnerCommand::Checkpoint(CheckpointRequest {
                completion: Some(ack_tx),
            }))
            .unwrap();
        let cp_result = ack_rx.recv_timeout(ACK_DEADLINE).unwrap();
        assert!(cp_result.is_ok(), "checkpoint should succeed");

        drop(assayer);
    }

    // Verify checkpoint file exists.
    assert!(cp_path.exists(), "checkpoint file should exist");

    // Step 2: Read checkpoint directly and validate.
    let payload = checkpoint::read_checkpoint(&cp_path).expect("should read checkpoint");
    assert_eq!(payload.last_processed_label_seq, 10);
    assert_eq!(payload.operational.parameters.p, 16);
    assert_eq!(payload.anchor.parameters.p, 15);
}

/// The checkpoint a deployment actually writes is whole-state: with no
/// payload constructed by hand, a running instance's acknowledged checkpoint
/// captures the Sentinel ledger it holds (a distinctive rate crossing value
/// for value), the identity dimension's published graph at the same energy
/// the live snapshot reports, the calibration buffer row for row, the Platt
/// tracker, the label counter and the health counters — and the rebuilt
/// instance serves that state: the restored ledger answers with the same
/// rate, and the first label after the restart counts from where the run
/// left off rather than from zero. This is the path the six hand-payload
/// tests establish capability for, exercised end to end
/// (´dec:durability:checkpoint-journal´).
///
/// ´claim:persistence:the-production-checkpoint-is-whole-state-and-the-restart-serves-it´
/// ´test:crate:checkpoint-whole-state-production-path´
#[test]
#[allow(clippy::too_many_lines)] // one deployment story told once, capture through restart
fn checkpoint_whole_state_production_path() {
    use crate::owner::commands::{CheckpointRequest as CpReq, IdentityDimensionRegistration};
    use crate::types::IdentityBudget;

    let dir = test_dir();
    let cp_path = dir.path().join("checkpoint.bin");

    let root_rate = 0.625_f64;
    let n_labels = 15_u64;
    let live_energy;

    {
        let assayer = test_assayer_with_persistence(dir.path(), "whole-state");
        assayer.register_sentinel(sentinel_reg(SentinelId(1))).unwrap();
        assayer
            .register_identity_dimension(IdentityDimensionRegistration {
                id: DimensionId(1),
                name: "ip-hash".to_owned(),
                description: "persisted test hierarchy".to_owned(),
                coordinate_semantics: "the first byte selects a prefix group".to_owned(),
                domain_bits: 128,
                depth_cutoff: 10,
                budget: IdentityBudget::for_depth_cutoff(10),
                encode: |entity| {
                    let bytes = entity.as_bytes();
                    bytes.first().map_or(0, |&b| u128::from(b) << 100)
                },
            })
            .unwrap();

        // Distinctive ledger state to watch across the boundary.
        {
            let lock = assayer
                .outcome_ledger
                .get(SentinelId(1))
                .expect("ledger created at registration");
            let mut guard = lock.write().unwrap();
            guard.root_mut().ewma_bad_rate = root_rate;
        }

        // Observations so the identity graph carries energy at capture.
        {
            let guard = assayer.identity_dimensions.read().unwrap();
            let infra = guard.get(&DimensionId(1)).expect("dimension registered");
            for i in 0..8_u128 {
                assert!(infra.observe(i << 100), "observation should enqueue");
            }
        }

        for i in 1..=n_labels {
            assayer.label_tx().send(dummy_label(AssessmentId(i), Some(i))).unwrap();
        }
        // Version 1 is construction, two lifecycle events bump to 3, then
        // one per label.
        let v = wait_for_version(assayer.shared(), 3 + n_labels, crate::testing::STATE_DEADLINE);
        assert!(v >= 3 + n_labels, "labels should be processed, version {v}");

        let (ack_tx, ack_rx) = crossbeam_channel::bounded(1);
        assayer
            .command_tx()
            .send(crate::owner::commands::ModelOwnerCommand::Checkpoint(CpReq {
                completion: Some(ack_tx),
            }))
            .unwrap();
        ack_rx.recv_timeout(ACK_DEADLINE).unwrap().expect("checkpoint should succeed");

        let snap_guard = assayer.identity_dimensions.read().unwrap();
        let infra = snap_guard.get(&DimensionId(1)).expect("dimension registered");
        let published = infra.graph_snapshot.load();
        live_energy = (**published)
            .as_ref()
            .expect("checkpoint published a graph snapshot")
            .total_energy();
        assert!(live_energy > 0, "observations should give the graph energy");
    }

    // Checkpoint out: the file a deployment writes carries the state.
    let payload = checkpoint::read_checkpoint(&cp_path).expect("read checkpoint");

    let (_, lp) = payload
        .ledger_state
        .iter()
        .find(|(id, _)| *id == SentinelId(1))
        .expect("the write site captures the Sentinel ledger");
    assert_eq!(lp.ledger.root().ewma_bad_rate, root_rate);

    let (_, ip) = payload
        .identity_state
        .iter()
        .find(|(id, _)| *id == DimensionId(1))
        .expect("the write site captures the identity dimension");
    assert_eq!(ip.graph_snapshot.total_energy(), live_energy);

    assert_eq!(payload.owner_state.label_count, n_labels);
    assert_eq!(
        payload.owner_state.calibration_buffer.len(),
        usize::try_from(n_labels).unwrap()
    );
    assert!(payload.owner_state.calibration_buffer.iter().all(|e| e.positive));
    // The two registrations mark an early Platt refit, which the first
    // label consumes and the refit resets the counter — so the tracker
    // crosses carrying one refit and the fourteen labels since it.
    assert_eq!(payload.owner_state.platt_tracker.refits_completed, 1);
    assert_eq!(
        payload.owner_state.platt_tracker.labels_since_refit,
        u32::try_from(n_labels - 1).unwrap()
    );
    assert_eq!(payload.owner_state.health_counters.eligible_labels, n_labels);
    assert_eq!(
        payload.owner_state.health_counters.labels_by_action.get(&Action::Allow),
        Some(&n_labels)
    );

    // Restore in: the rebuilt instance serves the captured state.
    let rebuilt = test_assayer_with_persistence(dir.path(), "whole-state-2");

    let lock = rebuilt
        .outcome_ledger
        .get(SentinelId(1))
        .expect("the restored ledger is attached to the runtime outcome ledger");
    let restored_rate = lock.read().unwrap().root().ewma_bad_rate;
    // Restore-decay for the milliseconds of downtime is the one
    // designed-in difference: stored averages age once at restore.
    assert!(
        (restored_rate - root_rate).abs() < 1e-6,
        "restored root rate {restored_rate} should be the captured {root_rate} less momentary decay"
    );

    // The owner-held counters continue rather than restart: the first
    // label after the restart is counted from the captured totals.
    rebuilt
        .label_tx()
        .send(dummy_label(AssessmentId(n_labels + 1), Some(n_labels + 1)))
        .unwrap();
    let deadline = std::time::Instant::now() + crate::testing::STATE_DEADLINE;
    loop {
        let summary = rebuilt.health_summary();
        if summary.total_labels == n_labels + 1 {
            assert_eq!(summary.eligible_labels, n_labels + 1);
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "restored counters should continue: total_labels {} after restart",
            summary.total_labels
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// The restored working copy and the replay list are consistent with each
/// other: the copy still reports the mark it was checkpointed at while the
/// entries handed back begin one past it. The caller replays onto a model that
/// has not yet advanced, so the mark and the list must describe the same
/// boundary from either side.
///
/// (´claim:persistence:only-entries-beyond-the-checkpoints-mark-are-handed-back-for-replay´)
/// ´test:crate:journal-replay-via-assayer-recovery´
#[test]
fn journal_replay_via_assayer_recovery() {
    let dir = test_dir();
    let cp_path = dir.path().join("checkpoint.bin");
    let jl_path = dir.path().join("journal.bin");

    // Create a working copy and manually advance it to seq=5.
    let mut working = WorkingCopy::cold_start(10, &super::helpers::test_model_config_with_anchor(5), 1000);
    working.last_processed_label_seq = 5;

    // Write checkpoint.
    let payload = working.to_checkpoint_payload(PersistentTimestamp::now(), Vec::new(), Vec::new());
    checkpoint::write_checkpoint(&cp_path, &payload).unwrap();

    // Write journal entries 1–10.
    {
        let mut file = journal::open_journal_for_append(&jl_path).unwrap();
        for i in 1..=10_u64 {
            let entry = JournalEntry {
                seq: i,
                label: test_label(AssessmentId(i)),
                context: test_pending_context(AssessmentId(i)),
                timestamp: PersistentTimestamp::now(),
            };
            journal::append_journal_entry(&mut file, &entry).unwrap();
        }
    }

    // Restore.
    let config = test_persistence_config(None);

    let result = recovery::attempt_restore(&cp_path, &jl_path, &config, &[], &[], PersistentTimestamp::now())
        .expect("the fixture checkpoint is not refused on its numbers");
    assert!(result.is_some(), "recovery should succeed");

    let result = result.unwrap();
    // Entries 1–5 skipped, 6–10 replayed.
    assert_eq!(result.replay_entries.len(), 5);
    assert_eq!(result.replay_entries[0].seq, 6);
    assert_eq!(result.replay_entries[4].seq, 10);
    assert_eq!(result.working.last_processed_label_seq, 5);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Assayer End-to-End Checkpoint
// ═══════════════════════════════════════════════════════════════════════════════

use crate::assessment::RequestContext;
use crate::config::types::InfrastructureConfig;
use crate::error::LabelError;
use crate::owner::commands::{CheckpointRequest, ModelOwnerCommand};
fn wp45_make_request(_channel: ChannelId, seed: u64) -> RequestContext {
    RequestContext::new(super::helpers::test_entity(seed))
}

fn wp45_make_label(assessment_id: AssessmentId, valence: f64) -> LabelData {
    LabelSpec::new(assessment_id).valence(valence).ground_truth().build()
}

/// An instance rebuilt from its own checkpoint comes up serving and keeps
/// learning from where it left off: it registers, assesses and returns a finite
/// risk on the first request, and ten further labels are all processed and
/// counted from the fifty the checkpoint captured rather than from zero.
/// Nothing about the restored state has to be re-taught before the model is
/// usable again — the point of persistence is that the restart is invisible
/// to the host on the other side of it.
///
/// ´claim:persistence:an-instance-rebuilt-from-its-checkpoint-serves-and-keeps-learning-immediately´
/// ´test:crate:checkpoint-round-trip-end-to-end´
#[test]
fn checkpoint_round_trip_end_to_end() {
    use crate::config::types::PersistenceConfig;

    let dir = test_dir();
    let cp_path = dir.path().join("checkpoint.bin");

    let persistence = PersistenceConfig {
        checkpoint_dir: dir.path().to_path_buf(),
        journal_dir: dir.path().to_path_buf(),
        checkpoint_interval: std::time::Duration::from_secs(3600),
    };

    // Step 1: Build, register sentinel, ingest report, reckon, label 50
    let _p_bad_before;
    {
        let config = AssayerConfig {
            instance_id: "wp45-checkpoint".to_owned(),
            infrastructure: InfrastructureConfig {
                label_channel_capacity: 100,
                ..crate::testing::test_infrastructure()
            },
            persistence: Some(persistence.clone()),
            ..Default::default()
        };

        let builder = Assayer::builder(config).signal_schema(&[]);
        let builder = builder.signal_schema(&[]);
        let assayer = builder.build().expect("test assayer should build");

        // Register + report
        assayer.register_sentinel(sentinel_reg(SentinelId(1))).unwrap();
        let report = super::helpers::test_report(&[(0x1000_0000_0000_0000_0000_0000_0000_0000, 8)]);
        assayer.receive_sentinel_report(SentinelId(1), report).unwrap();

        // Reckon and label 50 with 20% positive
        for i in 0_u64..50 {
            let results = assayer.derive_from_assess(&[wp45_make_request(ChannelId(0), i)]);
            let reckoning = results.into_iter().next().unwrap().unwrap();
            let valence = if i < 10 { 1.0 } else { -1.0 };
            assayer.label(wp45_make_label(reckoning.assessment.id, valence)).unwrap();
        }
        assayer.flush_label_channel().unwrap();

        // Record p_bad before checkpoint
        let results = assayer.derive_from_assess(&[wp45_make_request(ChannelId(0), 9999)]);
        _p_bad_before = results.into_iter().next().unwrap().unwrap().assessment.risk.p_bad;

        // Trigger explicit checkpoint
        let (ack_tx, ack_rx) = crossbeam_channel::bounded(1);
        assayer
            .command_tx()
            .send(ModelOwnerCommand::Checkpoint(CheckpointRequest {
                completion: Some(ack_tx),
            }))
            .unwrap();
        let cp_result = ack_rx.recv_timeout(ACK_DEADLINE).unwrap();
        assert!(cp_result.is_ok(), "checkpoint should succeed");

        drop(assayer);
    }

    // Verify checkpoint file exists
    assert!(cp_path.exists(), "checkpoint file should exist after drop");

    // Step 2: Rebuild with same config → verify checkpoint loaded
    {
        let config = AssayerConfig {
            instance_id: "wp45-checkpoint".to_owned(),
            infrastructure: InfrastructureConfig {
                label_channel_capacity: 100,
                ..crate::testing::test_infrastructure()
            },
            persistence: Some(persistence),
            ..Default::default()
        };

        let builder = Assayer::builder(config).signal_schema(&[]);
        let builder = builder.signal_schema(&[]);
        let assayer = builder.build().expect("restored assayer should build");

        // Assess → verify model state preserved (p̂ approximately same)
        let results = assayer.derive_from_assess(&[wp45_make_request(ChannelId(0), 9999)]);
        let p_bad_after = results.into_iter().next().unwrap().unwrap().assessment.risk.p_bad;

        // The model state should be approximately preserved. We allow some
        // drift from time decay but the general direction should match.
        assert!(p_bad_after.is_finite(), "restored p_bad should be finite");

        // Label 10 more → verify learning continues from checkpoint state
        for i in 0_u64..10 {
            let results = assayer.derive_from_assess(&[wp45_make_request(ChannelId(0), 50_000 + i)]);
            let reckoning = results.into_iter().next().unwrap().unwrap();
            let valence = if i < 2 { 1.0 } else { -1.0 };
            assayer.label(wp45_make_label(reckoning.assessment.id, valence)).unwrap();
        }
        assayer.flush_label_channel().unwrap();

        // Verify the model has processed all 10 labels, counted from the
        // 50 the checkpoint captured — the restored label counter
        // continues rather than restarting at zero.
        let summary = assayer.health_summary();
        assert_eq!(
            summary.total_labels, 60,
            "10 post-restore labels counted from the captured 50"
        );

        drop(assayer);
    }
}

/// The health a rebuilt instance reports is the health it had before shutdown:
/// window occupancy, total observations, calibrations completed and the current
/// thresholds all read the same on either side of the restart. Concordance is
/// how an operator judges whether the model is behaving, and a report that
/// silently reset at every deployment would make the metric mean nothing across
/// exactly the events most worth watching.
///
/// ´claim:persistence:the-health-a-rebuilt-instance-reports-is-the-health-it-had-before-shutdown´
/// ´test:crate:checkpoint-restores-concordance-tracker´
#[test]
fn checkpoint_restores_concordance_tracker() {
    use crate::config::types::PersistenceConfig;

    let dir = test_dir();
    let cp_path = dir.path().join("checkpoint.bin");

    let persistence = PersistenceConfig {
        checkpoint_dir: dir.path().to_path_buf(),
        journal_dir: dir.path().to_path_buf(),
        checkpoint_interval: std::time::Duration::from_secs(3600),
    };

    let config = AssayerConfig {
        instance_id: "wp45-concordance-cp".to_owned(),
        infrastructure: InfrastructureConfig {
            label_channel_capacity: 100,
            ..crate::testing::test_infrastructure()
        },
        concordance: crate::config::types::ConcordanceConfig {
            window_capacity: 6,
            recalibration_interval: 3,
            // A percentile is a fraction: the retired fixture value 80.0
            // sat outside the unit interval, and the builder now refuses
            // it — the exact defect the unreached-constraints entry named.
            percentile: 0.80,
        },
        persistence: Some(persistence),
        ..Default::default()
    };

    let before;
    {
        let assayer = Assayer::builder(config.clone())
            .signal_schema(&[])
            .build()
            .expect("assayer should build");

        for i in 0_u64..5 {
            drop(assayer.assess(&[wp45_make_request(ChannelId(0), i)]));
        }
        before = assayer.full_health_report().concordance;
        assert_eq!(before.window_size, 5);
        assert_eq!(before.total_observations, 5);
        assert_eq!(before.calibrations_completed, 1);

        let (ack_tx, ack_rx) = crossbeam_channel::bounded(1);
        assayer
            .command_tx()
            .send(ModelOwnerCommand::Checkpoint(CheckpointRequest {
                completion: Some(ack_tx),
            }))
            .unwrap();
        let cp_result = ack_rx.recv_timeout(ACK_DEADLINE).unwrap();
        assert!(cp_result.is_ok(), "checkpoint should succeed");

        drop(assayer);
    }

    assert!(cp_path.exists(), "checkpoint file should exist");

    {
        let assayer = Assayer::builder(config)
            .signal_schema(&[])
            .build()
            .expect("restored assayer should build");
        let after = assayer.full_health_report().concordance;

        assert_eq!(after.window_size, before.window_size);
        assert_eq!(after.total_observations, before.total_observations);
        assert_eq!(after.calibrations_completed, before.calibrations_completed);
        assert_eq!(after.current_thresholds, before.current_thresholds);

        drop(assayer);
    }
}

/// Shutting down immediately after a checkpoint completes is clean: the drop
/// returns rather than deadlocking against a model owner that has just finished
/// writing. This is the ordinary shape of a controlled restart — checkpoint,
/// then stop — so the two operations must not contend on the way past each
/// other.
///
/// (´claim:persistence:dropping-a-persistent-instance-unwinds-both-threads-without-hanging´)
/// ´test:crate:shutdown-after-checkpoint-trigger´
#[test]
fn shutdown_after_checkpoint_trigger() {
    use crate::config::types::PersistenceConfig;

    let dir = test_dir();

    let config = AssayerConfig {
        instance_id: "wp45-shutdown-cp".to_owned(),
        infrastructure: InfrastructureConfig {
            label_channel_capacity: 100,
            ..crate::testing::test_infrastructure()
        },
        persistence: Some(PersistenceConfig {
            checkpoint_dir: dir.path().to_path_buf(),
            journal_dir: dir.path().to_path_buf(),
            checkpoint_interval: std::time::Duration::from_secs(3600),
        }),
        ..Default::default()
    };

    let builder = Assayer::builder(config).signal_schema(&[]);
    let builder = builder.signal_schema(&[]);
    let assayer = builder.build().expect("assayer should build");

    // Trigger checkpoint
    let (ack_tx, ack_rx) = crossbeam_channel::bounded(1);
    assayer
        .command_tx()
        .send(ModelOwnerCommand::Checkpoint(CheckpointRequest {
            completion: Some(ack_tx),
        }))
        .unwrap();

    // Wait for checkpoint to complete
    drop(ack_rx.recv_timeout(ACK_DEADLINE));

    // Drop after checkpoint — should exit cleanly
    drop(assayer);
}

/// When the model owner cannot keep up, a journalling instance tells the host
/// precisely which of two things happened: the label was written down but not
/// handed on. It is never reported as a plain full channel, because the journal
/// mutex spans both the append and the hand-off, so any label that reached the
/// hand-off is already durable. The distinction is what the host retries on —
/// a durable label may be re-submitted safely, since replay will keep only the
/// first attempt.
///
/// ´claim:persistence:a-label-that-was-journaled-but-not-enqueued-is-reported-as-exactly-that´
/// ´test:crate:label-channel-full-with-journal´
#[test]
fn label_channel_full_with_journal() {
    use crate::config::types::PersistenceConfig;

    let dir = test_dir();

    // Build at the label channel's tabulated floor of one hundred slots
    // (´tab:config:concurrency´) with persistence enabled; the steward is
    // parked so the overflow is deterministic.
    let config = AssayerConfig {
        instance_id: "wp45-journal-full".to_owned(),
        infrastructure: InfrastructureConfig {
            label_channel_capacity: 100,
            ..crate::testing::test_infrastructure()
        },
        persistence: Some(PersistenceConfig {
            checkpoint_dir: dir.path().to_path_buf(),
            journal_dir: dir.path().to_path_buf(),
            checkpoint_interval: std::time::Duration::from_secs(3600),
        }),
        ..Default::default()
    };

    let builder = Assayer::builder(config).signal_schema(&[]);
    let assayer = builder.build().expect("assayer should build");

    // Reckon one more assessment than the channel holds.
    let mut assessment_ids = Vec::new();
    for i in 0_u64..101 {
        let results = assayer.derive_from_assess(&[wp45_make_request(ChannelId(0), i)]);
        let reckoning = results.into_iter().next().unwrap().unwrap();
        assessment_ids.push(reckoning.assessment.id);
    }

    // Park the steward so nothing drains while the channel fills.
    let (entered_tx, entered_rx) = crossbeam_channel::bounded(1);
    let (release_tx, release_rx) = crossbeam_channel::bounded(1);
    assayer
        .command_tx
        .send(crate::owner::commands::ModelOwnerCommand::TestBlock {
            entered: entered_tx,
            release: release_rx,
        })
        .expect("barrier command enqueues");
    entered_rx.recv_timeout(ACK_DEADLINE).expect("steward parks");

    // Submit labels past the channel's capacity — with journal, expect
    // `JournaledButNotEnqueued` (never `ChannelFull`, since the journal
    // mutex serialises append+try_send, so a journaled label that
    // reaches `try_send` is always classified as journaled — the
    // classification the submission surface promises
    // (´dec:surface:async-label´)).
    let mut got_journaled_not_enqueued = false;
    for &rid in &assessment_ids {
        match assayer.label(wp45_make_label(rid, 1.0)) {
            Ok(_) => {}
            Err(LabelError::JournaledButNotEnqueued) => {
                got_journaled_not_enqueued = true;
                break;
            }
            Err(e) => panic!("unexpected error: {e}"),
        }
    }

    assert!(
        got_journaled_not_enqueued,
        "should get JournaledButNotEnqueued once the declared capacity is exceeded"
    );

    release_tx.send(()).expect("steward releases");
}

/// Labels submitted from eight threads at once land in the journal in strictly
/// increasing sequence order, every one of them present, and the mark the model
/// owner subsequently checkpoints at is exactly the last sequence on disk. The
/// two facts together are what make replay safe: the ordering on disk matches
/// the order the owner received them, so the high-water mark cannot advance
/// past an entry that has not been applied. Were the mutex released between
/// writing a record and handing it on, a thread holding an earlier sequence
/// could be overtaken, the owner would raise its mark past that entry, and a
/// later crash would skip the overtaken label on replay without any trace of
/// having done so.
///
/// ´claim:persistence:concurrent-labelling-keeps-the-journal-order-and-the-owners-mark-in-step´
/// ´test:crate:concurrent-label-preserves-journal-channel-order´
#[test]
fn concurrent_label_preserves_journal_channel_order() {
    use std::sync::Arc;
    use std::thread;

    use crate::config::types::PersistenceConfig;
    use crate::persistence::journal;

    const THREADS: usize = 8;
    const PER_THREAD: u64 = 50;

    let dir = test_dir();
    let jl_path = dir.path().join("journal.bin");

    let config = AssayerConfig {
        instance_id: "concurrent-journal-order".to_owned(),
        infrastructure: InfrastructureConfig {
            // Generous capacity so try_send does not fail.
            label_channel_capacity: (THREADS as u64 * PER_THREAD * 4) as usize,
            ..crate::testing::test_infrastructure()
        },
        persistence: Some(PersistenceConfig {
            checkpoint_dir: dir.path().to_path_buf(),
            journal_dir: dir.path().to_path_buf(),
            checkpoint_interval: std::time::Duration::from_secs(3600),
        }),
        ..Default::default()
    };

    let builder = Assayer::builder(config).signal_schema(&[]);
    let builder = builder.signal_schema(&[]);
    let assayer = Arc::new(builder.build().expect("assayer should build"));

    // Pre-generate assessment IDs on one thread (assessment is cheap; we want
    // the labelling phase to be the contended one).
    let mut assessment_ids: Vec<AssessmentId> = Vec::with_capacity(THREADS * PER_THREAD as usize);
    for i in 0_u64..(THREADS as u64 * PER_THREAD) {
        let results = assayer.derive_from_assess(&[wp45_make_request(ChannelId(0), i)]);
        let reckoning = results.into_iter().next().unwrap().unwrap();
        assessment_ids.push(reckoning.assessment.id);
    }

    // Slice ids per thread (disjoint).
    let chunks: Vec<Vec<AssessmentId>> = assessment_ids
        .chunks(PER_THREAD as usize)
        .map(<[AssessmentId]>::to_vec)
        .collect();
    assert_eq!(chunks.len(), THREADS);

    let mut handles = Vec::with_capacity(THREADS);
    for chunk in chunks {
        let assayer = Arc::clone(&assayer);
        handles.push(thread::spawn(move || {
            for rid in chunk {
                assayer.label(wp45_make_label(rid, 1.0)).expect("label should succeed");
            }
        }));
    }
    for h in handles {
        h.join().expect("labeller thread panicked");
    }

    // Invariant 1: journal entries are written in strictly increasing
    // sequence order on disk. Read BEFORE the checkpoint below, which
    // truncates the journal (´dec:durability:checkpoint-journal´). All `label()` calls
    // have returned, so every journal entry is on disk.
    let entries = journal::read_journal(&jl_path).expect("read journal");
    let expected_count = THREADS * PER_THREAD as usize;
    assert_eq!(entries.len(), expected_count, "all labels should be journaled");
    for pair in entries.windows(2) {
        assert!(
            pair[1].seq > pair[0].seq,
            "journal sequence must be strictly increasing; got {} then {}",
            pair[0].seq,
            pair[1].seq
        );
    }
    let last_journal_seq = entries.last().unwrap().seq;

    // Request a synchronous checkpoint: this drains the label channel
    // on the model-owner thread (´dec:concurrency:single-steward´), then writes the
    // checkpoint and truncates the journal. Completion is the signal
    // that `last_processed_label_seq` reflects every submitted label.
    let (ack_tx, ack_rx) = crossbeam_channel::bounded(1);
    assayer
        .command_tx()
        .send(ModelOwnerCommand::Checkpoint(CheckpointRequest {
            completion: Some(ack_tx),
        }))
        .unwrap();
    ack_rx
        .recv_timeout(ACK_DEADLINE)
        .expect("checkpoint should complete")
        .expect("checkpoint should succeed");

    // Invariant 2: the model owner's last_processed_label_seq matches
    // the final journal sequence — no entry was skipped or re-ordered
    // past the high-water mark.
    let cp = crate::persistence::checkpoint::read_checkpoint(dir.path().join("checkpoint.bin").as_path())
        .expect("checkpoint should be readable");
    assert_eq!(
        cp.last_processed_label_seq, last_journal_seq,
        "model owner high-water mark ({}) must equal final journal seq ({})",
        cp.last_processed_label_seq, last_journal_seq
    );

    drop(assayer);
}

/// A checkpoint written before the rebuild's baseline existed carries none,
/// and the restore says so rather than inventing one: the model comes back
/// with the baseline absent, both shares of the precision matrix the prior
/// holds up at nothing, the coordinate-shaped share widened to the model's own
/// width, and the per-label check falling back to its counter because it has
/// nothing to be relative to. The first rebuild after the restore establishes
/// the record, at which point every relative arm becomes available again. A
/// restore that fabricated a baseline would put the check to work against a
/// state no rebuild had measured. The two components of the synchronisation
/// reading come back absent for the same reason and by two different routes:
/// the record that would carry them is not there, and the share of the clamp
/// mass the covariance has not been rebuilt against is not persisted at all,
/// because it is a fact about a pair rather than about a model and the pair
/// the checkpoint restores is one the rebuild already agreed with. A restored
/// model therefore attributes none of its drift to the prior until it has
/// rebuilt once, which errs toward the cadence reacting to drift it cannot
/// account for rather than away from it.
///
/// ´claim:persistence:a-checkpoint-without-a-baseline-restores-without-one-and-the-first-rebuild-establishes-it´
/// ´test:crate:a-checkpoint-without-a-baseline-restores-without-one´
#[test]
fn a_checkpoint_without_a_baseline_restores_without_one() {
    use crate::linalg::convert::vec_to_col;
    use crate::model::bayesian::BayesianLinearModel;
    use crate::model::recompute::{
        DEFAULT_KAPPA_GROWTH_FACTOR, RecomputeTrigger, VisitInputs, should_recompute, synchronisation_error_threshold,
        synchronisation_visit,
    };

    let p = 6;
    let live = BayesianLinearModel::new(p, 0.1, 1000);
    let written = live.to_checkpoint_state();

    // What a checkpoint from before this layout carries: every field the older
    // format had, and none of the four it did not. Serde's defaults are what
    // an older payload's missing fields decode to, and this is that state
    // written out by hand rather than inferred.
    let older = crate::persistence::checkpoint::CheckpointModelState {
        parameters: written.parameters,
        precision: written.precision,
        labels_since_recompute: 0,
        n_recompute_effective: 1000,
        consecutive_clean_recomputes: 0,
        last_diagonal_ratio: 1.0,
        last_sync_error: 0.0,
        sync_error_shortenings: 0,
        total_recomputes: 0,
        measurements: 0,
        floored_rebuilds: 0,
        cascade_terminus_events: 0,
        last_dimensions_at_floor: 0,
        alarms: 0,
        baseline: None,
        spectral_floor_mass: 0.0,
        clamp_mass: Vec::new(),
    };

    let mu = vec_to_col(&older.parameters.mu);
    let covariance = crate::linalg::symmetric::SymmetricMatrix::from_computation(crate::linalg::convert::vec_to_mat(
        &older.parameters.covariance_data,
        older.parameters.p,
        older.parameters.p,
    ));
    let mut restored = BayesianLinearModel::from_checkpoint(
        mu,
        older.precision,
        covariance,
        0.1,
        crate::types::ModelId::Operational,
        older.labels_since_recompute,
        older.n_recompute_effective,
        older.consecutive_clean_recomputes,
        older.last_diagonal_ratio,
        older.last_sync_error,
        older.sync_error_shortenings,
        older.total_recomputes,
        older.measurements,
        older.floored_rebuilds,
        older.cascade_terminus_events,
        older.last_dimensions_at_floor,
        older.alarms,
        older.baseline,
        older.spectral_floor_mass,
        older.clamp_mass,
    )
    .expect("a prior-initialised precision matrix factors");

    assert!(restored.baseline().is_none(), "the restore reports the absence");
    assert!(
        (restored.spectral_floor_mass() - 0.0).abs() < f64::EPSILON,
        "no spectral floor was accumulated before the restore"
    );
    assert_eq!(
        restored.clamp_mass().len(),
        p,
        "the empty coordinate-shaped share widens to the model's width"
    );
    assert!(
        restored.clamp_mass().iter().all(|&m| m == 0.0),
        "and every coordinate carries nothing"
    );
    assert_eq!(
        restored.clamp_mass_since_rebuild().len(),
        p,
        "the share the covariance has not been rebuilt against widens with it"
    );
    assert!(
        restored.clamp_mass_since_rebuild().iter().all(|&m| m == 0.0),
        "and starts at nothing, because a restored model has no disagreement to attribute until it rebuilds"
    );
    assert!(
        restored.last_prior_induced_sync_error().abs() < f64::EPSILON && restored.last_sync_error_residual().abs() < f64::EPSILON,
        "and both components of the reading are absent along with the record that would carry them"
    );

    // Only the counter is available, so a model part way through its interval
    // is left alone whatever its conditioning reads.
    let quiet = should_recompute(
        restored.precision(),
        500,
        1000,
        restored.baseline(),
        DEFAULT_KAPPA_GROWTH_FACTOR,
        0.001,
    );
    assert!(matches!(quiet, RecomputeTrigger::NotNeeded { .. }), "got {quiet:?}");

    // The first rebuild after the restore establishes the record.
    let trigger = should_recompute(
        restored.precision(),
        1000,
        1000,
        restored.baseline(),
        DEFAULT_KAPPA_GROWTH_FACTOR,
        0.001,
    );
    assert!(matches!(trigger, RecomputeTrigger::Counter { .. }), "got {trigger:?}");
    let inputs = VisitInputs::from_trigger(&trigger, 1000, synchronisation_error_threshold(p), 0.0, &[]);
    let (outcome, new_covariance, baseline) = synchronisation_visit(
        restored.precision(),
        restored.covariance(),
        restored.baseline(),
        true,
        &inputs,
    );
    restored.apply_recompute_outcome(&outcome, new_covariance, Some(baseline), 1000);
    assert!(restored.baseline().is_some(), "the first rebuild establishes the record");
    assert!(
        restored
            .baseline()
            .and_then(crate::model::recompute::RecomputeBaseline::spectrum)
            .is_some(),
        "and with it the spectrum the floor is derived from"
    );
}

/// The restore reads the spectrum where the storage boundary read only the
/// diagonal, and the witness is the suite's own: ones on the diagonal and twos
/// off it, whose minimum diagonal is strictly positive and whose least
/// eigenvalue is minus one. Both halves stand here rather than only the
/// second, because a witness that failed the diagonal test would witness
/// nothing — the whole content of the repair is that the cheap test says yes
/// exactly where the restore now says no. The refusal names the pivot the
/// factorisation stopped at, so a reader inspecting the artefact is pointed at
/// a coordinate rather than at a file.
///
/// ´claim:persistence:a-checkpoint-precision-matrix-is-refused-on-its-spectrum-not-its-diagonal´
/// ´test:crate:checkpoint-restore-refuses-the-indefinite-witness´
#[test]
fn checkpoint_restore_refuses_the_indefinite_witness() {
    use crate::error::BuildError;
    use crate::linalg::symmetric::SymmetricMatrix;
    use crate::model::bayesian::BayesianLinearModel;
    use crate::types::ModelId;

    let p = 4;
    let mut data = faer::Mat::zeros(p, p);
    for i in 0..p {
        data[(i, i)] = 1.0;
    }
    data[(0, 1)] = 2.0;
    data[(1, 0)] = 2.0;
    let precision = SymmetricMatrix::from_computation(data);

    // Red: every check standing between the file and the model admits it.
    let (_, min_diagonal) = precision.diagonal_min_max();
    assert!(
        min_diagonal > 0.0,
        "the witness must pass the diagonal test, or it witnesses nothing (got {min_diagonal})"
    );

    // Green: the restore refuses it, and says where.
    let restored = BayesianLinearModel::from_checkpoint(
        faer::Col::zeros(p),
        precision,
        SymmetricMatrix::identity_scaled(p, 1.0),
        0.1,
        ModelId::Sister,
        0,
        1000,
        0,
        0.0,
        0.0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        None,
        0.0,
        Vec::new(),
    );
    // `BayesianLinearModel` carries no `Debug`, so the refusal is taken by
    // pattern rather than by `expect_err`.
    let Err(refusal) = restored else {
        panic!("a precision matrix with a negative eigenvalue must be refused");
    };

    match refusal {
        BuildError::CheckpointNotPositiveDefinite { model, pivot } => {
            assert_eq!(model, ModelId::Sister, "the refusal names the model it was taken for");
            assert_eq!(
                pivot, 1,
                "the second pivot is the one the factorisation cannot take: 1 - 2² is negative"
            );
        }
        other => panic!("the refusal must be the checkpoint's own variant, got {other:?}"),
    }
}

/// A checkpoint the model may hold restores through the same call untouched:
/// the verdict is a gate and not a transformation. The mean, the precision and
/// the covariance come back bit for bit, and the cadence fields come back as
/// the values that were stored rather than as the defaults a fresh model
/// would carry. Asserting the restore still works is not redundant beside the
/// refusal — a verdict that refused everything would satisfy the refusal test
/// alone.
///
/// ´claim:persistence:the-definiteness-verdict-passes-a-healthy-checkpoint-through-unchanged´
/// ´test:crate:checkpoint-restore-admits-a-healthy-precision-matrix´
#[test]
fn checkpoint_restore_admits_a_healthy_precision_matrix() {
    use crate::linalg::symmetric::SymmetricMatrix;
    use crate::model::bayesian::BayesianLinearModel;
    use crate::types::ModelId;

    let p = 4;
    let precision = SymmetricMatrix::identity_scaled(p, 3.0);
    let covariance = SymmetricMatrix::identity_scaled(p, 1.0 / 3.0);
    #[allow(clippy::cast_precision_loss)] // Justified: tiny test index
    let mu = faer::Col::from_fn(p, |i| i as f64 + 0.5);

    let restored = BayesianLinearModel::from_checkpoint(
        mu.clone(),
        precision.clone(),
        covariance.clone(),
        0.1,
        ModelId::Operational,
        7,
        1000,
        3,
        1.25,
        0.5,
        2,
        11,
        0,
        0,
        0,
        0,
        0,
        None,
        0.0,
        Vec::new(),
    )
    .expect("a scaled identity is positive definite");

    for i in 0..p {
        assert_eq!(
            restored.mu()[i].to_bits(),
            mu[i].to_bits(),
            "the mean crosses the verdict unchanged at {i}"
        );
        assert_eq!(
            restored.precision().diagonal_element(i).to_bits(),
            precision.diagonal_element(i).to_bits(),
            "the precision crosses the verdict unchanged at {i}"
        );
        assert_eq!(
            restored.covariance().diagonal_element(i).to_bits(),
            covariance.diagonal_element(i).to_bits(),
            "the covariance crosses the verdict unchanged at {i}"
        );
    }
    assert_eq!(restored.labels_since_recompute(), 7, "the stored cadence count is kept");
    assert_eq!(restored.total_recomputes(), 11, "the stored lifetime count is kept");
}

/// A checkpoint that is read, structurally agreed and then refused on its
/// numbers fails the restore rather than becoming a cold start, and the error
/// is the builder's own. The distinction is the whole of the repair: an
/// unreadable or structurally incompatible checkpoint is a deployment with
/// nothing to resume and cold-starting costs it nothing it had, while a
/// checkpoint whose precision matrix the model may not hold is corruption, and
/// cold-starting on it would discard every learned parameter at the one moment
/// the host could still reach for an earlier checkpoint or the journal.
///
/// ´claim:persistence:a-checkpoint-refused-on-its-numbers-fails-the-restore-rather-than-cold-starting´
/// ´test:crate:recovery-refuses-an-indefinite-checkpoint´
#[test]
fn recovery_refuses_an_indefinite_checkpoint() {
    use crate::error::BuildError;
    use crate::linalg::symmetric::SymmetricMatrix;
    use crate::types::ModelId;

    let dir = test_dir();
    let cp_path = dir.path().join("checkpoint.bin");
    let jl_path = dir.path().join("journal.bin");

    let working = test_working_copy();
    let mut payload = working.to_checkpoint_payload(PersistentTimestamp::now(), Vec::new(), Vec::new());

    // The suite's witness widened to the sister model's own width: symmetric
    // to the bit, every diagonal entry strictly positive, and one eigenvalue
    // at minus one. It crosses the storage boundary untouched, because that
    // boundary's only verdict is symmetry.
    let p = payload.sister.parameters.p;
    let mut data = faer::Mat::zeros(p, p);
    for i in 0..p {
        data[(i, i)] = 1.0;
    }
    data[(0, 1)] = 2.0;
    data[(1, 0)] = 2.0;
    payload.sister.precision = SymmetricMatrix::from_computation(data);

    checkpoint::write_checkpoint(&cp_path, &payload).expect("write failed");
    fs::write(&jl_path, b"").unwrap();

    let config = test_persistence_config(None);
    // `RecoveryResult` carries no `Debug`, so the refusal is taken by pattern
    // rather than by `expect_err`.
    let Err(refusal) = recovery::attempt_restore(&cp_path, &jl_path, &config, &[], &[], PersistentTimestamp::now()) else {
        panic!("a checkpoint carrying an indefinite precision matrix must not be a cold start");
    };

    match refusal {
        BuildError::CheckpointNotPositiveDefinite { model, pivot } => {
            assert_eq!(model, ModelId::Sister, "the refusal names the model whose matrix it read");
            assert_eq!(pivot, 1, "and the pivot the factorisation stopped at");
        }
        other => panic!("the refusal must be the checkpoint's own variant, got {other:?}"),
    }
}

/// The repaired count crosses the checkpoint, and it crosses it in the slot the retired repair cascade's own count occupied. The payload's encoding is positional rather than named, so the rename moved no byte and the layout generation did not turn on it (the suite's version-mismatch test pins the generation at what it was); what a reader gets back is the count the model had. The residual this leaves is stated at the record rather than hidden here: a checkpoint written before the counter's subject moved restores its old reading into the new field, which is admitted because both readings answer the one question the counter has always asked (´dec:durability:structural-compatibility´).
///
/// ´claim:persistence:the-repaired-count-crosses-the-checkpoint-in-the-slot-the-retired-count-occupied´
/// ´test:crate:the-repaired-count-crosses-the-checkpoint´
#[test]
fn the_repaired_count_crosses_the_checkpoint() {
    use crate::linalg::symmetric::SymmetricMatrix;
    use crate::model::bayesian::BayesianLinearModel;
    use crate::model::recompute::{VisitInputs, synchronisation_error_threshold, synchronisation_visit};
    use crate::types::ModelId;

    let p = 6;

    // A weakest direction far below the floor, so the rebuild is a repair and
    // the count is a measurement rather than a planted constant.
    let precision = SymmetricMatrix::from_computation(faer::Mat::from_fn(p, p, |i, j| {
        if i != j {
            0.0
        } else if i == 0 {
            300.0
        } else if i == 1 {
            1e-18
        } else {
            1.0
        }
    }));
    let covariance = SymmetricMatrix::identity_scaled(p, 1.0);
    let inputs = VisitInputs {
        labels_since_recompute: 1000,
        diagonal_ratio: 1.0,
        dimensions_at_floor: 0,
        sync_error_threshold: synchronisation_error_threshold(p),
        spectral_floor_mass: 0.0,
        clamp_mass: &[],
    };
    let (outcome, rebuilt, baseline) = synchronisation_visit(&precision, &covariance, None, true, &inputs);
    assert!(baseline.floor_repaired(), "the rebuild had a floor to apply");

    let mut live = BayesianLinearModel::new(p, 0.1, 1_000_000);
    live.apply_recompute_outcome(&outcome, rebuilt, Some(baseline), 1_000_000);
    assert_eq!(live.floored_rebuilds(), 1, "the model counted the repair");

    let written = live.to_checkpoint_state();
    assert_eq!(written.floored_rebuilds, 1, "and the checkpoint state carries it");

    let mu = crate::linalg::convert::vec_to_col(&written.parameters.mu);
    let restored_covariance = SymmetricMatrix::from_computation(crate::linalg::convert::vec_to_mat(
        &written.parameters.covariance_data,
        written.parameters.p,
        written.parameters.p,
    ));
    let restored = BayesianLinearModel::from_checkpoint(
        mu,
        written.precision,
        restored_covariance,
        0.1,
        ModelId::Operational,
        written.labels_since_recompute,
        written.n_recompute_effective,
        written.consecutive_clean_recomputes,
        written.last_diagonal_ratio,
        written.last_sync_error,
        written.sync_error_shortenings,
        written.total_recomputes,
        written.measurements,
        written.floored_rebuilds,
        written.cascade_terminus_events,
        written.last_dimensions_at_floor,
        written.alarms,
        written.baseline,
        written.spectral_floor_mass,
        written.clamp_mass,
    )
    .expect("the floored precision matrix factors, which is what the floor is for");

    assert_eq!(restored.floored_rebuilds(), 1, "and the restore recovers it");
    assert_eq!(
        restored.total_recomputes(),
        live.total_recomputes(),
        "beside the counter it has always sat next to",
    );
}

/// Writes `payload`, reads it back, and returns the refusal the reconstruction
/// answers the read-back payload with, as the four figures the refusal names.
///
/// The round trip is the point rather than a convenience. It puts the damaged
/// payload through every proof the reader takes — the magic, the format
/// version and the checksum over the whole payload — so a refusal obtained
/// afterwards is a refusal of bytes that are, by the reader's own account,
/// exactly the bytes that were written.
fn refusal_after_round_trip(
    payload: &CheckpointPayload,
    path: &std::path::Path,
) -> (crate::types::ModelId, String, usize, usize) {
    checkpoint::write_checkpoint(path, payload).expect("write failed");
    let read_back = checkpoint::read_checkpoint(path).expect("the damaged payload passes magic, version and checksum");
    let refusal = WorkingCopy::from_checkpoint_payload(&read_back, 100, 0.1)
        .expect_err("a state that disagrees with its own width is refused rather than reconstructed");
    match refusal {
        crate::error::BuildError::CheckpointDimensionMismatch {
            model,
            quantity,
            found,
            declared,
        } => (model, quantity.to_owned(), found, declared),
        other => panic!("the restore names the disagreement rather than something else: {other}"),
    }
}

/// A checkpoint whose model state declares one width and carries a mean, a
/// precision matrix, a covariance or a clamp mass of another is refused as an
/// error, and the refusal names the model and both disagreeing extents. The
/// checksum the reader takes is an integrity check and not a consistency one,
/// and the structural check taken after it compares the artefact against this
/// build rather than against itself, so an internally inconsistent state
/// passes both; the conversion that widens the flat covariance back into a
/// matrix on the declared width then asserts in every build, and a defective
/// artefact ends the process at startup instead of leaving the host the three
/// actions the refusal exists to leave it (´dec:degradation:error-partition´).
///
/// ´claim:persistence:a-model-state-that-disagrees-with-its-own-declared-width-is-refused-rather-than-aborting´
/// ´test:crate:a-model-state-that-disagrees-with-its-own-width-is-refused´
#[test]
fn a_model_state_that_disagrees_with_its_own_width_is_refused() {
    use crate::linalg::symmetric::SymmetricMatrix;
    use crate::types::ModelId;

    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");
    let working = test_working_copy();

    // The control: undamaged, the same payload restores.
    let intact = test_payload(&working);
    checkpoint::write_checkpoint(&path, &intact).expect("write failed");
    let read_back = checkpoint::read_checkpoint(&path).expect("read failed");
    WorkingCopy::from_checkpoint_payload(&read_back, 100, 0.1).expect("a state consistent with itself still restores");

    // One covariance entry short of the declared width squared — the smallest
    // damage the conversion cannot survive.
    let mut short_covariance = test_payload(&working);
    let declared_p = short_covariance.operational.parameters.p;
    short_covariance.operational.parameters.covariance_data.pop();
    assert_eq!(
        refusal_after_round_trip(&short_covariance, &path),
        (
            ModelId::Operational,
            "covariance entries".to_owned(),
            declared_p * declared_p - 1,
            declared_p
        ),
    );

    // A mean one entry longer than the width it is the mean of. Nothing
    // asserts on this one: the reconstruction reads the model's width off the
    // mean and the declared width off the parameters, and a model holding a
    // mean and a covariance of different widths is what would be built.
    let mut long_mean = test_payload(&working);
    long_mean.sister.parameters.mu.push(0.0);
    assert_eq!(
        refusal_after_round_trip(&long_mean, &path),
        (ModelId::Sister, "mean entries".to_owned(), declared_p + 1, declared_p),
    );

    // A precision matrix that is square and symmetric — everything the
    // storage boundary checks about it — and of the wrong width.
    let mut narrow_precision = test_payload(&working);
    let anchor_p = narrow_precision.anchor.parameters.p;
    narrow_precision.anchor.precision = SymmetricMatrix::identity_scaled(anchor_p - 1, 1.0);
    assert_eq!(
        refusal_after_round_trip(&narrow_precision, &path),
        (ModelId::Anchor, "precision rows".to_owned(), anchor_p - 1, anchor_p),
    );

    // A clamp mass that is present and of another width. An absent one is a
    // reading a state written before the coordinate-shaped mass existed does
    // not have, and widens to zeros; a present one of the wrong width is a
    // reading that disagrees with the model it was taken on, and widening it
    // would discard it in silence.
    let mut wrong_clamp = test_payload(&working);
    wrong_clamp.operational.clamp_mass = vec![0.0; declared_p - 1];
    assert_eq!(
        refusal_after_round_trip(&wrong_clamp, &path),
        (
            ModelId::Operational,
            "clamp-mass entries".to_owned(),
            declared_p - 1,
            declared_p
        ),
    );
}

/// The width check reaches every registered outcome axis and not only the
/// three fixed models: an axis state one covariance entry short of its own
/// declared width is refused, and the refusal names that axis. The axes are
/// the part of the payload whose membership the host decides at runtime, so
/// they are both the part a checkpoint carries an unbounded number of and the
/// part a check written against the fixed three would silently not cover.
///
/// ´claim:persistence:the-width-check-covers-every-outcome-axis-and-not-only-the-three-fixed-models´
/// ´test:crate:an-outcome-axis-state-that-disagrees-with-its-own-width-is-refused´
#[test]
fn an_outcome_axis_state_that_disagrees_with_its_own_width_is_refused() {
    use crate::snapshot::working::WorkingAxisModel;
    use crate::types::{ModelId, OutcomeEligibility};

    let dir = test_dir();
    let path = dir.path().join("checkpoint.bin");

    let axis_p = 8;
    let mut working = test_working_copy();
    working.outcome_models.insert(
        OutcomeAxisId(7),
        WorkingAxisModel {
            name: "chargeback".to_owned(),
            description: "probability the payment is later reversed".to_owned(),
            model: crate::model::bayesian::BayesianLinearModel::new(axis_p, 0.1, 1000),
            kappa_a: 2.0,
            gamma: 0.9997,
            spatial: false,
            eligibility: OutcomeEligibility::AllLabels,
        },
    );

    // The control: the axis restores when its own extents agree.
    let intact = test_payload(&working);
    checkpoint::write_checkpoint(&path, &intact).expect("write failed");
    let read_back = checkpoint::read_checkpoint(&path).expect("read failed");
    let restored = WorkingCopy::from_checkpoint_payload(&read_back, 100, 0.1).expect("a consistent axis state still restores");
    assert_eq!(restored.outcome_models[&OutcomeAxisId(7)].model.dim(), axis_p);

    let mut damaged = test_payload(&working);
    damaged.outcome_models[0].1.model.parameters.covariance_data.pop();
    assert_eq!(
        refusal_after_round_trip(&damaged, &path),
        (
            ModelId::OutcomeAxis(OutcomeAxisId(7)),
            "covariance entries".to_owned(),
            axis_p * axis_p - 1,
            axis_p
        ),
    );
}
