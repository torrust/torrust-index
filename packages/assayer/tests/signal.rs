// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`scalar_width_is_one`] | signal | A scalar signal occupies exactly one column of the feature vector. Widths are what the schema computes its offsets from, so a shape that turns one number into one number must claim one position and no more. |
//! | [`log_scaled_width_is_one`] | signal | cites (´claim:signal:a-single-valued-shape-occupies-exactly-one-feature-column´) |
//! | [`binary_width_is_one`] | signal | cites (´claim:signal:a-single-valued-shape-occupies-exactly-one-feature-column´) |
//! | [`ordinal_width_is_one`] | signal | cites (´claim:signal:a-single-valued-shape-occupies-exactly-one-feature-column´) |
//! | [`cyclic_width_is_two`] | signal | A cyclic signal claims two columns, because it is carried as the sine and cosine of its phase rather than as a bare number. A single column could not represent a wrap-around without a discontinuity at the seam. |
//! | [`hashed_categorical_width_matches`] | signal | A hashed categorical claims exactly as many columns as it declares bins, whatever that number is. The declared width is the layout itself, not a hint the encoder is free to round. |
//! | [`vector_width_matches_len`] | signal | A vector signal claims one column per declared element, independently of the clipping bounds it carries: widening or narrowing the bounds moves no offsets. |
//! | [`encodes_value_within_clip`] | signal | A scalar already inside its declared bounds reaches the feature vector untouched. The clip is a guard against out-of-range hosts, not a rescaling that ordinary values have to pay for. |
//! | [`clamps_to_lower_bound`] | signal | A scalar below its declared floor arrives as the floor. A host cannot push a feature outside the range the model was built to expect, however wrong the number it supplies. |
//! | [`clamps_to_upper_bound`] | signal | cites (´claim:signal:a-scalar-outside-its-clip-arrives-at-the-nearest-bound´) |
//! | [`sanitises_nan`] | signal | A NaN scalar encodes as zero rather than as NaN. One unusable number would otherwise spread through every sum and product downstream of it, so it is neutralised at the boundary where it enters. |
//! | [`sanitises_infinity`] | signal | cites (´claim:signal:a-non-finite-scalar-encodes-as-zero-rather-than-entering-the-feature-vector´) |
//! | [`sanitises_neg_infinity`] | signal | cites (´claim:signal:a-non-finite-scalar-encodes-as-zero-rather-than-entering-the-feature-vector´) |
//! | [`encodes_positive_value`] | signal | The log-scaled shape reports the natural logarithm of one plus the value over its divisor, so a value equal to its divisor becomes the logarithm of two. Quantities spanning orders of magnitude are compressed into a range the model can weigh, instead of one large count dominating every other feature. |
//! | [`encodes_negative_value`] | signal | Compression is applied to the magnitude and the sign restored afterwards, so a value and its negation encode to opposite numbers of equal size. Direction survives the transform that flattens scale. |
//! | [`encodes_zero`] | signal | Zero passes through the log scaling as zero, so an absent or neutral count sits at the origin rather than at some offset the model would have to learn around. |
//! | [`below_threshold_is_zero`] | signal | The binary shape splits at one half, and anything short of a half encodes as zero — a value only just below the line is treated as firmly as one far below it, because the shape carries a decision rather than a degree. |
//! | [`at_threshold_is_one`] | signal | cites (´claim:signal:the-binary-shape-splits-at-one-half-with-the-threshold-itself-counting-as-set´) |
//! | [`above_threshold_is_one`] | signal | cites (´claim:signal:the-binary-shape-splits-at-one-half-with-the-threshold-itself-counting-as-set´) |
//! | [`normalises_value`] | signal | An ordinal is divided by its declared maximum, so half of the scale reaches the model as a half whatever the underlying units were. Signals with quite different ranges therefore arrive on a common footing. |
//! | [`clamps_above_max`] | signal | A value beyond the declared maximum arrives as one rather than as something greater than one: a host that under-declares its range cannot push an ordinal feature off the unit interval. |
//! | [`clamps_below_zero`] | signal | cites (´claim:signal:an-ordinal-outside-its-declared-range-is-held-inside-the-unit-interval´) |
//! | [`encodes_quarter_period`] | signal | A cyclic value is carried as the sine and cosine of its phase: a quarter of the way round the period lands at sine one, cosine zero. The pair is what lets the model see that the end of a cycle sits next to its beginning. |
//! | [`cyclic_encodes_zero`] | signal | cites (´claim:signal:a-cyclic-value-is-carried-as-the-sine-and-cosine-of-its-phase´) |
//! | [`encodes_half_period`] | signal | cites (´claim:signal:a-cyclic-value-is-carried-as-the-sine-and-cosine-of-its-phase´) |
//! | [`produces_one_hot_vector`] | signal | A category sets exactly one of the declared bins to one and leaves every other at zero, so the encoding is a genuine one-hot: no single category can contribute weight in two places at once. |
//! | [`different_strings_different_indices`] | signal | Distinct categories reach distinct bins when the declared width is generous — across a thousand bins two ordinary words do not collide. Collision is a consequence of narrowing the width, not something the encoding imposes. |
//! | [`same_string_produces_same_index`] | signal | The same category always reaches the same bin, so a category means the same thing to the model on every request. The hashing here is a stable layout device rather than a randomisation. |
//! | [`handles_zero_width`] | signal | A categorical declared with no bins encodes to an empty vector rather than indexing into one that does not exist. A degenerate declaration costs nothing and crashes nothing. |
//! | [`encodes_full_vector`] | signal | A vector of exactly the declared length passes through element by element and in order, so the model's columns line up with the host's components. |
//! | [`clamps_elements`] | signal | Clipping bounds apply to each element separately: one component below the floor and another above the ceiling are each brought to their own bound while the one between them is left alone. The vector is never rescaled as a whole, so a single wild component cannot distort its neighbours. |
//! | [`pads_short_vector_with_zeros`] | signal | A vector shorter than declared is padded with zeros out to the declared width. The schema's offsets are fixed, so a host supplying too few components displaces nothing that follows. |
//! | [`truncates_long_vector`] | signal | A vector longer than declared is cut to the declared width, keeping the leading components. Surplus components are dropped rather than allowed to spill into the next signal's columns. |
//! | [`categorical_with_scalar_produces_zeros`] | signal | A value of the wrong kind for its declared shape encodes as zeros of that shape's own width instead of failing. The layout stays intact, so one mis-typed signal degrades to no evidence rather than corrupting the vector or refusing the request. |
//! | [`numeric_with_hashed_categorical_produces_zeros`] | signal | cites (´claim:signal:a-value-of-the-wrong-kind-encodes-as-zeros-of-the-declared-width-rather-than-failing´) |
//! | [`creates_entity_persistent`] | signal | A declaration keeps the name and persistence mode it was given, unaltered. These are what the schema keys its layout and its cache mask on, so they have to survive construction verbatim. |
//! | [`creates_request_scoped`] | signal | cites (´claim:signal:a-declaration-keeps-the-name-and-persistence-mode-it-was-given´) |
//! | [`empty_declarations`] | signal | A schema built from no declarations has no width and no signals. An installation that declares nothing is a legitimate configuration rather than an error to be reported. |
//! | [`single_scalar`] | signal | A schema's total width is the sum of its declarations' feature widths, and its signal count is the number of declarations: one one-wide signal gives a width of one. |
//! | [`multiple_scalars`] | signal | cites (´claim:signal:a-schemas-total-width-is-the-sum-of-its-declarations-feature-widths´) |
//! | [`mixed_widths`] | signal | cites (´claim:signal:a-schemas-total-width-is-the-sum-of-its-declarations-feature-widths´) |
//! | [`rejects_duplicate_names`] | signal | Two declarations sharing a name are refused while the schema is being built, and the error names the colliding signal. Offsets are looked up by name, so a duplicate would silently make one of the two unreachable — better to fail at build time than to lose a signal at runtime. |
//! | [`encodes_single_signal`] | signal | Encoding places each supplied signal's value at the offset the schema assigned it, so what the model reads at a column is what the host supplied for the signal declared there. |
//! | [`unprovided_signal_is_zero`] | signal | A signal the host did not supply reads as zero in its own column while the signals around it stay where they were. Absence is encoded in place rather than by omission, so a sparse request produces a vector of the same shape as a complete one. |
//! | [`ignores_unknown_signals`] | signal | A name the schema never declared is ignored: the encoded vector keeps the schema's width and carries only declared signals. A host that sends more than was agreed cannot widen or reorder the feature space. |
//! | [`encodes_multi_width_signals`] | signal | A multi-column signal fills every column of its own span: a cyclic signal declared after a scalar writes its sine into the second position and its cosine into the third. Wide signals occupy contiguous runs, which is what makes offsets computed from widths address them correctly. |
//! | [`empty_schema_produces_empty_vector`] | signal | A schema with no declarations encodes to an empty vector, so the zero- signal configuration flows through the encoding path without needing a special case. |
//! | [`empty_schema`] | signal | An empty schema has an empty persistence mask, matching its zero width. The mask and the vector it describes always have the same length, so indexing one by the other is safe at every size. |
//! | [`all_entity_persistent`] | signal | The persistence mask reads true at exactly the positions belonging to entity-persistent signals. The cache uses it to decide which columns to remember, so it has to describe positions rather than names. |
//! | [`all_request_scoped`] | signal | cites (´claim:signal:the-persistence-mask-reads-true-at-exactly-the-entity-persistent-positions´) |
//! | [`mixed_persistence`] | signal | cites (´claim:signal:the-persistence-mask-reads-true-at-exactly-the-entity-persistent-positions´) |
//! | [`multi_width_signal_mask`] | signal | A wide signal's persistence covers every one of its columns: an entity- persistent cyclic signal marks both its sine and its cosine position. Remembering half of such a pair would leave the model reading a phase that never occurred. |
//! | [`miss_returns_zeros_without_signals`] | signal | An entity the cache has never seen, asked about with no signals at all, reads as zeros across the schema's full width. A cold entity yields a well-formed vector of no evidence rather than a short one or none. |
//! | [`hit_returns_cached_entity_persistent`] | signal | An entity-persistent value supplied once comes back on later requests that do not resupply it. That is what entity persistence is for: an expensive or occasional signal keeps informing the model between the requests that can actually produce it. |
//! | [`request_scoped_not_cached`] | signal | A request-scoped value is visible in the request that supplied it and gone from the next, which reads zero. Signals describing this request and not this entity must not be attributed to the entity afterwards. |
//! | [`merge_entity_and_request`] | signal | The merged vector layers this request's signals over the entity's remembered ones: a request supplying only its own short-lived signal still reads the persistent one recorded earlier. Neither kind of evidence has to be resupplied in order to keep the other. |
//! | [`cache_write_is_deferred_until_the_drain`] | signal | The cache write is deferred off the merging call: the call that supplies an entity-persistent value returns it merged, but the cached copy is untouched until the deferred write is drained — a read before the drain finds nothing, a read after it finds the value. The merge is thereby a read plus an enqueue, which is what keeps the assessment path's write set the three enumerated writes and its additions enqueues rather than mutations; the drain runs on the identity maintenance thread's cycle in production. |
//! | [`update_overwrites_cached_value`] | signal | Resupplying an entity-persistent signal replaces what was remembered rather than blending with it, so the cache holds the latest reading and not an average over a history the host never asked it to keep. |
//! | [`different_entities_are_independent`] | signal | Each entity's remembered signals are its own: writing one entity's value leaves another's untouched, and both read back exactly what they were given. Without that, one entity's history would become evidence against another. |
//! | [`evicts_oldest_at_capacity`] | signal | A cache at capacity admits a new entity by dropping exactly one existing one, and counts the drop. The evicted entity is not an error afterwards — it simply reads as a cold miss again, so a bounded cache degrades to forgetting rather than to failing. |
//! | [`evicts_lru_not_just_oldest`] | signal | Reading an entity renews it, so eviction falls on the least recently used rather than the first inserted: an entity read just before the cache filled survives while an older untouched one goes. Retention therefore follows live traffic instead of arrival order. |
//! | [`tracks_hits_and_misses`] | signal | Every access is counted as exactly one hit or one miss, the first access that stores a value included — that one is a miss, because it found nothing. A hit rate therefore reads against every access the cache ever served, not against the ones after it warmed. |
//! | [`health_reports_correct_capacity_and_size`] | signal | Health reports the configured capacity alongside the number of entities actually held, and that count tracks insertions. An operator can see how close a cache is running to its ceiling before evictions start, not only after. |
//! | [`hit_rate_calculation`] | signal | The hit rate is hits over all accesses, hits and misses together — not hits against misses, and not against capacity. |
//! | [`hit_rate_zero_when_no_accesses`] | signal | A cache that has never been accessed reports a hit rate of zero rather than a division by nothing, so a freshly started process can be scraped for metrics before it has served anything. |
//! | [`utilisation_calculation`] | signal | Utilisation is the number of entities held over the configured capacity, so a half-full cache reads as a half regardless of what its hit and miss counters say. |
//! | [`nan_signal_produces_zero`] | signal | cites (´claim:signal:a-non-finite-scalar-encodes-as-zero-rather-than-entering-the-feature-vector´) |
//! | [`cached_nan_is_sanitised`] | signal | What the cache stores is the sanitised encoding and not the raw value, so an unusable number cannot be read back on a later request either. A single bad reading is neutralised once rather than remembered and re- served for the life of the entry. |
//! | [`empty_schema_returns_empty_vector`] | signal | A cache over a zero-width schema returns an empty vector rather than a vector of zeros. The degenerate configuration — nothing declared at all — travels the ordinary merge path and needs no special case at the call site. |
//! | [`concurrent_get_merge_no_panics`] | signal | Four threads merging four hundred distinct entities at once neither panic nor lose an access: each one is counted as a miss and every entity ends up held. Concurrency here costs contention on the store's lock, not correctness of what the store contains. |
//! | [`concurrent_get_merge_with_contention`] | signal | When four threads contend on the same ten entities, the hit and miss counters still sum to exactly the thousand accesses made, the store settles at one record per entity, and nothing is evicted because capacity was never approached. How the total splits between hits and misses is left to the race; the total itself is not. |

#![allow(clippy::float_cmp)]

//! Integration tests for `torrust_assayer::signal`.
//!
//! Tests the public API of the signal module
//! (´chap:spec:host-signals´): encoding, schema, and cache.

use std::collections::HashMap;
use std::sync::Arc;

use torrust_assayer::signal_export::{SignalCache, SignalCacheHealth, SignalSchemaIndex, encode_signal};
use torrust_assayer::testing::World;
use torrust_assayer::testing::signals::scalar_decl;
use torrust_assayer::types::EntityKey;
use torrust_assayer::{Persistence, SignalDeclaration, SignalShape, SignalValue};

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Shorthand for [`World::entity`] — the canonical name→`EntityKey` mapping
/// used throughout the integration-test harness.
fn entity_key(s: &str) -> EntityKey {
    World::entity(s)
}

fn build_schema(decls: &[SignalDeclaration]) -> Arc<SignalSchemaIndex> {
    Arc::new(SignalSchemaIndex::from_declarations(decls).unwrap())
}

// ═══════════════════════════════════════════════════════════════════════════════
// SignalShape::feature_width
// ═══════════════════════════════════════════════════════════════════════════════

mod feature_width {
    use super::*;

    /// A scalar signal occupies exactly one column of the feature vector.
    /// Widths are what the schema computes its offsets from, so a shape that
    /// turns one number into one number must claim one position and no more.
    ///
    /// ´claim:signal:a-single-valued-shape-occupies-exactly-one-feature-column´
    /// ´test:integration:scalar-width-is-one´
    #[test]
    fn scalar_width_is_one() {
        assert_eq!(SignalShape::Scalar { clip: (0.0, 1.0) }.feature_width(), 1);
    }

    /// Compressing a value through a logarithm does not change how much room
    /// it needs: a log-scaled signal still claims one column, because the
    /// transform rewrites the number rather than expanding it.
    ///
    /// (´claim:signal:a-single-valued-shape-occupies-exactly-one-feature-column´)
    /// ´test:integration:log-scaled-width-is-one´
    #[test]
    fn log_scaled_width_is_one() {
        assert_eq!(SignalShape::LogScaled { divisor: 1.0 }.feature_width(), 1);
    }

    /// A binary indicator is carried as a single zero-or-one column rather
    /// than as a two-position one-hot pair, so the schema spends one feature
    /// on it.
    ///
    /// (´claim:signal:a-single-valued-shape-occupies-exactly-one-feature-column´)
    /// ´test:integration:binary-width-is-one´
    #[test]
    fn binary_width_is_one() {
        assert_eq!(SignalShape::Binary.feature_width(), 1);
    }

    /// An ordinal is carried as one normalised number rather than as a column
    /// per rung, so however large its declared maximum it still claims one
    /// position.
    ///
    /// (´claim:signal:a-single-valued-shape-occupies-exactly-one-feature-column´)
    /// ´test:integration:ordinal-width-is-one´
    #[test]
    fn ordinal_width_is_one() {
        assert_eq!(SignalShape::Ordinal { max: 10.0 }.feature_width(), 1);
    }

    /// A cyclic signal claims two columns, because it is carried as the sine
    /// and cosine of its phase rather than as a bare number. A single column
    /// could not represent a wrap-around without a discontinuity at the seam.
    ///
    /// ´claim:signal:a-cyclic-shape-occupies-two-feature-columns-for-its-sine-and-cosine´
    /// ´test:integration:cyclic-width-is-two´
    #[test]
    fn cyclic_width_is_two() {
        assert_eq!(SignalShape::Cyclic { period: 24.0 }.feature_width(), 2);
    }

    /// A hashed categorical claims exactly as many columns as it declares
    /// bins, whatever that number is. The declared width is the layout itself,
    /// not a hint the encoder is free to round.
    ///
    /// ´claim:signal:a-hashed-categorical-shape-occupies-exactly-its-declared-bin-count´
    /// ´test:integration:hashed-categorical-width-matches´
    #[test]
    fn hashed_categorical_width_matches() {
        assert_eq!(SignalShape::HashedCategorical { width: 4 }.feature_width(), 4);
        assert_eq!(SignalShape::HashedCategorical { width: 8 }.feature_width(), 8);
        assert_eq!(SignalShape::HashedCategorical { width: 16 }.feature_width(), 16);
    }

    /// A vector signal claims one column per declared element, independently
    /// of the clipping bounds it carries: widening or narrowing the bounds
    /// moves no offsets.
    ///
    /// ´claim:signal:a-vector-shape-occupies-exactly-its-declared-element-count´
    /// ´test:integration:vector-width-matches-len´
    #[test]
    fn vector_width_matches_len() {
        assert_eq!(
            SignalShape::Vector {
                len: 5,
                clip: (0.0, 1.0)
            }
            .feature_width(),
            5
        );
        assert_eq!(
            SignalShape::Vector {
                len: 10,
                clip: (-1.0, 1.0)
            }
            .feature_width(),
            10
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// encode_signal: Scalar
// ═══════════════════════════════════════════════════════════════════════════════

mod encode_scalar {
    use super::*;

    /// A scalar already inside its declared bounds reaches the feature vector
    /// untouched. The clip is a guard against out-of-range hosts, not a
    /// rescaling that ordinary values have to pay for.
    ///
    /// ´claim:signal:a-scalar-inside-its-clip-passes-through-unchanged´
    /// ´test:integration:encodes-value-within-clip´
    #[test]
    fn encodes_value_within_clip() {
        let shape = SignalShape::Scalar { clip: (0.0, 10.0) };
        let encoded = encode_signal(&SignalValue::Numeric(3.5), &shape);
        assert_eq!(encoded, vec![3.5]);
    }

    /// A scalar below its declared floor arrives as the floor. A host cannot
    /// push a feature outside the range the model was built to expect, however
    /// wrong the number it supplies.
    ///
    /// ´claim:signal:a-scalar-outside-its-clip-arrives-at-the-nearest-bound´
    /// ´test:integration:clamps-to-lower-bound´
    #[test]
    fn clamps_to_lower_bound() {
        let shape = SignalShape::Scalar { clip: (0.0, 10.0) };
        let encoded = encode_signal(&SignalValue::Numeric(-5.0), &shape);
        assert_eq!(encoded, vec![0.0]);
    }

    /// The same guard holds at the other end: a value ten times the ceiling is
    /// delivered as the ceiling rather than as an outlier at a scale the model
    /// has never seen.
    ///
    /// (´claim:signal:a-scalar-outside-its-clip-arrives-at-the-nearest-bound´)
    /// ´test:integration:clamps-to-upper-bound´
    #[test]
    fn clamps_to_upper_bound() {
        let shape = SignalShape::Scalar { clip: (0.0, 10.0) };
        let encoded = encode_signal(&SignalValue::Numeric(100.0), &shape);
        assert_eq!(encoded, vec![10.0]);
    }

    /// A NaN scalar encodes as zero rather than as NaN. One unusable number
    /// would otherwise spread through every sum and product downstream of it,
    /// so it is neutralised at the boundary where it enters.
    ///
    /// ´claim:signal:a-non-finite-scalar-encodes-as-zero-rather-than-entering-the-feature-vector´
    /// ´test:integration:sanitises-nan´
    #[test]
    fn sanitises_nan() {
        let shape = SignalShape::Scalar { clip: (0.0, 10.0) };
        let encoded = encode_signal(&SignalValue::Numeric(f64::NAN), &shape);
        assert_eq!(encoded, vec![0.0]);
    }

    /// Infinity is treated as unusable rather than as a very large number: it
    /// encodes as zero, not as the clip's upper bound. Sanitisation runs
    /// before clamping, so an infinite input is not quietly converted into a
    /// legitimate-looking maximum.
    ///
    /// (´claim:signal:a-non-finite-scalar-encodes-as-zero-rather-than-entering-the-feature-vector´)
    /// ´test:integration:sanitises-infinity´
    #[test]
    fn sanitises_infinity() {
        let shape = SignalShape::Scalar { clip: (0.0, 10.0) };
        let encoded = encode_signal(&SignalValue::Numeric(f64::INFINITY), &shape);
        assert_eq!(encoded, vec![0.0]);
    }

    /// Negative infinity is neutralised alongside its positive twin, so
    /// neither end of the extended real line can reach the model as a feature.
    ///
    /// (´claim:signal:a-non-finite-scalar-encodes-as-zero-rather-than-entering-the-feature-vector´)
    /// ´test:integration:sanitises-neg-infinity´
    #[test]
    fn sanitises_neg_infinity() {
        let shape = SignalShape::Scalar { clip: (0.0, 10.0) };
        let encoded = encode_signal(&SignalValue::Numeric(f64::NEG_INFINITY), &shape);
        assert_eq!(encoded, vec![0.0]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// encode_signal: LogScaled
// ═══════════════════════════════════════════════════════════════════════════════

mod encode_log_scaled {
    use super::*;

    /// The log-scaled shape reports the natural logarithm of one plus the
    /// value over its divisor, so a value equal to its divisor becomes the
    /// logarithm of two. Quantities spanning orders of magnitude are
    /// compressed into a range the model can weigh, instead of one large count
    /// dominating every other feature.
    ///
    /// ´claim:signal:the-log-scaled-shape-compresses-magnitude-through-the-logarithm-of-one-plus-the-ratio´
    /// ´test:integration:encodes-positive-value´
    #[test]
    fn encodes_positive_value() {
        let shape = SignalShape::LogScaled { divisor: 1.0 };
        let encoded = encode_signal(&SignalValue::Numeric(1.0), &shape);
        // ln(1 + 1/1) * 1 = ln(2)
        assert!((encoded[0] - 2_f64.ln()).abs() < 1e-10);
    }

    /// Compression is applied to the magnitude and the sign restored
    /// afterwards, so a value and its negation encode to opposite numbers of
    /// equal size. Direction survives the transform that flattens scale.
    ///
    /// ´claim:signal:the-log-scaled-shape-preserves-the-sign-of-the-magnitude-it-compresses´
    /// ´test:integration:encodes-negative-value´
    #[test]
    fn encodes_negative_value() {
        let shape = SignalShape::LogScaled { divisor: 1.0 };
        let encoded = encode_signal(&SignalValue::Numeric(-1.0), &shape);
        // ln(1 + 1/1) * -1 = -ln(2)
        assert!((encoded[0] + 2_f64.ln()).abs() < 1e-10);
    }

    /// Zero passes through the log scaling as zero, so an absent or neutral
    /// count sits at the origin rather than at some offset the model would
    /// have to learn around.
    ///
    /// ´claim:signal:a-log-scaled-zero-encodes-as-zero´
    /// ´test:integration:encodes-zero´
    #[test]
    fn encodes_zero() {
        let shape = SignalShape::LogScaled { divisor: 1.0 };
        let encoded = encode_signal(&SignalValue::Numeric(0.0), &shape);
        assert_eq!(encoded, vec![0.0]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// encode_signal: Binary
// ═══════════════════════════════════════════════════════════════════════════════

mod encode_binary {
    use super::*;

    /// The binary shape splits at one half, and anything short of a half
    /// encodes as zero — a value only just below the line is treated as firmly
    /// as one far below it, because the shape carries a decision rather than a
    /// degree.
    ///
    /// ´claim:signal:the-binary-shape-splits-at-one-half-with-the-threshold-itself-counting-as-set´
    /// ´test:integration:below-threshold-is-zero´
    #[test]
    fn below_threshold_is_zero() {
        let encoded = encode_signal(&SignalValue::Numeric(0.49), &SignalShape::Binary);
        assert_eq!(encoded, vec![0.0]);
    }

    /// The boundary itself belongs to the upper side: exactly one half encodes
    /// as one. A host emitting a bare probability of a half gets a definite
    /// answer rather than one that depends on which way the comparison was
    /// written.
    ///
    /// (´claim:signal:the-binary-shape-splits-at-one-half-with-the-threshold-itself-counting-as-set´)
    /// ´test:integration:at-threshold-is-one´
    #[test]
    fn at_threshold_is_one() {
        let encoded = encode_signal(&SignalValue::Numeric(0.5), &SignalShape::Binary);
        assert_eq!(encoded, vec![1.0]);
    }

    /// Anything past the boundary encodes as one regardless of how far past,
    /// so the magnitude of a binary signal above its threshold carries no
    /// weight.
    ///
    /// (´claim:signal:the-binary-shape-splits-at-one-half-with-the-threshold-itself-counting-as-set´)
    /// ´test:integration:above-threshold-is-one´
    #[test]
    fn above_threshold_is_one() {
        let encoded = encode_signal(&SignalValue::Numeric(0.51), &SignalShape::Binary);
        assert_eq!(encoded, vec![1.0]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// encode_signal: Ordinal
// ═══════════════════════════════════════════════════════════════════════════════

mod encode_ordinal {
    use super::*;

    /// An ordinal is divided by its declared maximum, so half of the scale
    /// reaches the model as a half whatever the underlying units were. Signals
    /// with quite different ranges therefore arrive on a common footing.
    ///
    /// ´claim:signal:an-ordinal-is-divided-by-its-declared-maximum-onto-the-unit-interval´
    /// ´test:integration:normalises-value´
    #[test]
    fn normalises_value() {
        let shape = SignalShape::Ordinal { max: 10.0 };
        let encoded = encode_signal(&SignalValue::Numeric(5.0), &shape);
        assert_eq!(encoded, vec![0.5]);
    }

    /// A value beyond the declared maximum arrives as one rather than as
    /// something greater than one: a host that under-declares its range cannot
    /// push an ordinal feature off the unit interval.
    ///
    /// ´claim:signal:an-ordinal-outside-its-declared-range-is-held-inside-the-unit-interval´
    /// ´test:integration:clamps-above-max´
    #[test]
    fn clamps_above_max() {
        let shape = SignalShape::Ordinal { max: 10.0 };
        let encoded = encode_signal(&SignalValue::Numeric(15.0), &shape);
        assert_eq!(encoded, vec![1.0]);
    }

    /// The same holds below: a negative ordinal arrives as zero, since the
    /// encoding expresses a position along a declared scale and there is no
    /// position before that scale begins.
    ///
    /// (´claim:signal:an-ordinal-outside-its-declared-range-is-held-inside-the-unit-interval´)
    /// ´test:integration:clamps-below-zero´
    #[test]
    fn clamps_below_zero() {
        let shape = SignalShape::Ordinal { max: 10.0 };
        let encoded = encode_signal(&SignalValue::Numeric(-5.0), &shape);
        assert_eq!(encoded, vec![0.0]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// encode_signal: Cyclic
// ═══════════════════════════════════════════════════════════════════════════════

mod encode_cyclic {
    use super::*;

    /// A cyclic value is carried as the sine and cosine of its phase: a
    /// quarter of the way round the period lands at sine one, cosine zero. The
    /// pair is what lets the model see that the end of a cycle sits next to
    /// its beginning.
    ///
    /// ´claim:signal:a-cyclic-value-is-carried-as-the-sine-and-cosine-of-its-phase´
    /// ´test:integration:encodes-quarter-period´
    #[test]
    fn encodes_quarter_period() {
        let shape = SignalShape::Cyclic { period: 1.0 };
        let encoded = encode_signal(&SignalValue::Numeric(0.25), &shape);
        // phase = 2π * 0.25 / 1 = π/2
        // sin(π/2) = 1, cos(π/2) = 0
        assert!((encoded[0] - 1.0).abs() < 1e-10);
        assert!(encoded[1].abs() < 1e-10);
    }

    /// The start of a cycle is a specific point on the circle rather than a
    /// degenerate case: it lands at sine zero, cosine one.
    ///
    /// (´claim:signal:a-cyclic-value-is-carried-as-the-sine-and-cosine-of-its-phase´)
    /// ´test:integration:cyclic-encodes-zero´
    #[test]
    fn cyclic_encodes_zero() {
        let shape = SignalShape::Cyclic { period: 1.0 };
        let encoded = encode_signal(&SignalValue::Numeric(0.0), &shape);
        // sin(0) = 0, cos(0) = 1
        assert!(encoded[0].abs() < 1e-10);
        assert!((encoded[1] - 1.0).abs() < 1e-10);
    }

    /// Half a period round lands diametrically opposite the start, at cosine
    /// minus one. Points far apart in the cycle are far apart in the encoding,
    /// which a plain remainder of the raw value would not have given.
    ///
    /// (´claim:signal:a-cyclic-value-is-carried-as-the-sine-and-cosine-of-its-phase´)
    /// ´test:integration:encodes-half-period´
    #[test]
    fn encodes_half_period() {
        let shape = SignalShape::Cyclic { period: 1.0 };
        let encoded = encode_signal(&SignalValue::Numeric(0.5), &shape);
        // phase = π => sin(π) = 0, cos(π) = -1
        assert!(encoded[0].abs() < 1e-10);
        assert!((encoded[1] + 1.0).abs() < 1e-10);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// encode_signal: HashedCategorical
// ═══════════════════════════════════════════════════════════════════════════════

mod encode_hashed_categorical {
    use super::*;

    /// A category sets exactly one of the declared bins to one and leaves
    /// every other at zero, so the encoding is a genuine one-hot: no single
    /// category can contribute weight in two places at once.
    ///
    /// ´claim:signal:a-category-sets-exactly-one-bin-and-leaves-the-rest-at-zero´
    /// ´test:integration:produces-one-hot-vector´
    #[test]
    fn produces_one_hot_vector() {
        let shape = SignalShape::HashedCategorical { width: 8 };
        let encoded = encode_signal(&SignalValue::Categorical("cat".to_string()), &shape);
        assert_eq!(encoded.len(), 8);

        // Exactly one element should be 1.0
        let ones_count = encoded.iter().filter(|&&v| v == 1.0).count();
        assert_eq!(ones_count, 1);

        // All others should be 0.0
        let zeros_count = encoded.iter().filter(|&&v| v == 0.0).count();
        assert_eq!(zeros_count, 7);
    }

    /// Distinct categories reach distinct bins when the declared width is
    /// generous — across a thousand bins two ordinary words do not collide.
    /// Collision is a consequence of narrowing the width, not something the
    /// encoding imposes.
    ///
    /// ´claim:signal:distinct-categories-reach-distinct-bins-when-the-declared-width-is-generous´
    /// ´test:integration:different-strings-different-indices´
    #[test]
    fn different_strings_different_indices() {
        let shape = SignalShape::HashedCategorical { width: 1000 };
        let encoded1 = encode_signal(&SignalValue::Categorical("cat".to_string()), &shape);
        let encoded2 = encode_signal(&SignalValue::Categorical("dog".to_string()), &shape);

        // Find the indices
        let idx1 = encoded1.iter().position(|&v| v == 1.0).unwrap();
        let idx2 = encoded2.iter().position(|&v| v == 1.0).unwrap();

        // With width 1000, "cat" and "dog" should hash to different indices
        assert_ne!(idx1, idx2);
    }

    /// The same category always reaches the same bin, so a category means the
    /// same thing to the model on every request. The hashing here is a stable
    /// layout device rather than a randomisation.
    ///
    /// ´claim:signal:the-same-category-always-reaches-the-same-bin´
    /// ´test:integration:same-string-produces-same-index´
    #[test]
    fn same_string_produces_same_index() {
        let shape = SignalShape::HashedCategorical { width: 8 };
        let encoded1 = encode_signal(&SignalValue::Categorical("test".to_string()), &shape);
        let encoded2 = encode_signal(&SignalValue::Categorical("test".to_string()), &shape);
        assert_eq!(encoded1, encoded2);
    }

    /// A categorical declared with no bins encodes to an empty vector rather
    /// than indexing into one that does not exist. A degenerate declaration
    /// costs nothing and crashes nothing.
    ///
    /// ´claim:signal:a-zero-width-categorical-encodes-to-nothing-instead-of-indexing-into-an-empty-vector´
    /// ´test:integration:handles-zero-width´
    #[test]
    fn handles_zero_width() {
        let shape = SignalShape::HashedCategorical { width: 0 };
        let encoded = encode_signal(&SignalValue::Categorical("cat".to_string()), &shape);
        assert_eq!(encoded, [] as [f64; 0]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// encode_signal: Vector
// ═══════════════════════════════════════════════════════════════════════════════

mod encode_vector {
    use super::*;

    /// A vector of exactly the declared length passes through element by
    /// element and in order, so the model's columns line up with the host's
    /// components.
    ///
    /// ´claim:signal:a-vector-of-the-declared-length-passes-through-element-by-element´
    /// ´test:integration:encodes-full-vector´
    #[test]
    fn encodes_full_vector() {
        let shape = SignalShape::Vector {
            len: 3,
            clip: (0.0, 10.0),
        };
        let encoded = encode_signal(&SignalValue::Vector(vec![1.0, 2.0, 3.0]), &shape);
        assert_eq!(encoded, vec![1.0, 2.0, 3.0]);
    }

    /// Clipping bounds apply to each element separately: one component below
    /// the floor and another above the ceiling are each brought to their own
    /// bound while the one between them is left alone. The vector is never
    /// rescaled as a whole, so a single wild component cannot distort its
    /// neighbours.
    ///
    /// ´claim:signal:vector-clipping-is-applied-to-each-element-separately´
    /// ´test:integration:clamps-elements´
    #[test]
    fn clamps_elements() {
        let shape = SignalShape::Vector {
            len: 3,
            clip: (0.0, 5.0),
        };
        let encoded = encode_signal(&SignalValue::Vector(vec![-1.0, 3.0, 10.0]), &shape);
        assert_eq!(encoded, vec![0.0, 3.0, 5.0]);
    }

    /// A vector shorter than declared is padded with zeros out to the declared
    /// width. The schema's offsets are fixed, so a host supplying too few
    /// components displaces nothing that follows.
    ///
    /// ´claim:signal:a-short-vector-is-padded-with-zeros-to-the-declared-width´
    /// ´test:integration:pads-short-vector-with-zeros´
    #[test]
    fn pads_short_vector_with_zeros() {
        let shape = SignalShape::Vector {
            len: 5,
            clip: (0.0, 10.0),
        };
        let encoded = encode_signal(&SignalValue::Vector(vec![1.0, 2.0]), &shape);
        assert_eq!(encoded, vec![1.0, 2.0, 0.0, 0.0, 0.0]);
    }

    /// A vector longer than declared is cut to the declared width, keeping the
    /// leading components. Surplus components are dropped rather than allowed
    /// to spill into the next signal's columns.
    ///
    /// ´claim:signal:a-long-vector-is-truncated-to-the-declared-width´
    /// ´test:integration:truncates-long-vector´
    #[test]
    fn truncates_long_vector() {
        let shape = SignalShape::Vector {
            len: 2,
            clip: (0.0, 10.0),
        };
        let encoded = encode_signal(&SignalValue::Vector(vec![1.0, 2.0, 3.0, 4.0]), &shape);
        assert_eq!(encoded, vec![1.0, 2.0]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// encode_signal: Type Mismatches
// ═══════════════════════════════════════════════════════════════════════════════

mod mismatched_types {
    use super::*;

    /// A value of the wrong kind for its declared shape encodes as zeros of
    /// that shape's own width instead of failing. The layout stays intact, so
    /// one mis-typed signal degrades to no evidence rather than corrupting the
    /// vector or refusing the request.
    ///
    /// ´claim:signal:a-value-of-the-wrong-kind-encodes-as-zeros-of-the-declared-width-rather-than-failing´
    /// ´test:integration:categorical-with-scalar-produces-zeros´
    #[test]
    fn categorical_with_scalar_produces_zeros() {
        let shape = SignalShape::Scalar { clip: (0.0, 10.0) };
        let encoded = encode_signal(&SignalValue::Categorical("cat".to_string()), &shape);
        assert_eq!(encoded, vec![0.0]);
    }

    /// The fallback holds in the other direction and at multi-column widths
    /// too: a number offered where a category was declared yields the whole
    /// run of zeros the schema had reserved for it.
    ///
    /// (´claim:signal:a-value-of-the-wrong-kind-encodes-as-zeros-of-the-declared-width-rather-than-failing´)
    /// ´test:integration:numeric-with-hashed-categorical-produces-zeros´
    #[test]
    fn numeric_with_hashed_categorical_produces_zeros() {
        let shape = SignalShape::HashedCategorical { width: 4 };
        let encoded = encode_signal(&SignalValue::Numeric(42.0), &shape);
        assert_eq!(encoded, vec![0.0, 0.0, 0.0, 0.0]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SignalDeclaration
// ═══════════════════════════════════════════════════════════════════════════════

mod signal_declaration {
    use super::*;

    /// A declaration keeps the name and persistence mode it was given,
    /// unaltered. These are what the schema keys its layout and its cache mask
    /// on, so they have to survive construction verbatim.
    ///
    /// ´claim:signal:a-declaration-keeps-the-name-and-persistence-mode-it-was-given´
    /// ´test:integration:creates-entity-persistent´
    #[test]
    fn creates_entity_persistent() {
        let decl = SignalDeclaration::new("auth_score", SignalShape::Scalar { clip: (0.0, 1.0) }, Persistence::Entity);
        assert_eq!(decl.name, "auth_score");
        assert_eq!(decl.persistence, Persistence::Entity);
    }

    /// The same holds for a request-scoped declaration: being short-lived
    /// changes nothing about how the declaration records itself.
    ///
    /// (´claim:signal:a-declaration-keeps-the-name-and-persistence-mode-it-was-given´)
    /// ´test:integration:creates-request-scoped´
    #[test]
    fn creates_request_scoped() {
        let decl = SignalDeclaration::new("ip_reputation", SignalShape::Binary, Persistence::Request);
        assert_eq!(decl.name, "ip_reputation");
        assert_eq!(decl.persistence, Persistence::Request);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SignalSchemaIndex::from_declarations
// ═══════════════════════════════════════════════════════════════════════════════

mod from_declarations {
    use torrust_assayer::error::BuildError;

    use super::*;

    /// A schema built from no declarations has no width and no signals. An
    /// installation that declares nothing is a legitimate configuration rather
    /// than an error to be reported.
    ///
    /// ´claim:signal:a-schema-with-no-declarations-has-no-width-and-no-signals´
    /// ´test:integration:empty-declarations´
    #[test]
    fn empty_declarations() {
        let schema = SignalSchemaIndex::from_declarations(&[]).unwrap();
        assert_eq!(schema.total_width(), 0);
        assert_eq!(schema.signal_count(), 0);
    }

    /// A schema's total width is the sum of its declarations' feature widths,
    /// and its signal count is the number of declarations: one one-wide signal
    /// gives a width of one.
    ///
    /// ´claim:signal:a-schemas-total-width-is-the-sum-of-its-declarations-feature-widths´
    /// ´test:integration:single-scalar´
    #[test]
    fn single_scalar() {
        let decls = vec![scalar_decl("a", Persistence::Entity)];
        let schema = SignalSchemaIndex::from_declarations(&decls).unwrap();
        assert_eq!(schema.total_width(), 1);
        assert_eq!(schema.signal_count(), 1);
    }

    /// Several one-wide signals accumulate to a width equal to their number,
    /// whatever mixture of persistence modes they carry — persistence governs
    /// caching, not layout.
    ///
    /// (´claim:signal:a-schemas-total-width-is-the-sum-of-its-declarations-feature-widths´)
    /// ´test:integration:multiple-scalars´
    #[test]
    fn multiple_scalars() {
        let decls = vec![
            scalar_decl("a", Persistence::Entity),
            scalar_decl("b", Persistence::Request),
            scalar_decl("c", Persistence::Entity),
        ];
        let schema = SignalSchemaIndex::from_declarations(&decls).unwrap();
        assert_eq!(schema.total_width(), 3);
        assert_eq!(schema.signal_count(), 3);
    }

    /// Widths of different sizes accumulate the same way: a scalar, a binary
    /// and a two-column cyclic come to four. Signals are laid out back to back
    /// with no padding between them.
    ///
    /// (´claim:signal:a-schemas-total-width-is-the-sum-of-its-declarations-feature-widths´)
    /// ´test:integration:mixed-widths´
    #[test]
    fn mixed_widths() {
        let decls = vec![
            SignalDeclaration::new("scalar", SignalShape::Scalar { clip: (0.0, 1.0) }, Persistence::Entity),
            SignalDeclaration::new("binary", SignalShape::Binary, Persistence::Entity),
            SignalDeclaration::new("cyclic", SignalShape::Cyclic { period: 24.0 }, Persistence::Request),
        ];
        let schema = SignalSchemaIndex::from_declarations(&decls).unwrap();
        // 1 + 1 + 2 = 4
        assert_eq!(schema.total_width(), 4);
    }

    /// Two declarations sharing a name are refused while the schema is being
    /// built, and the error names the colliding signal. Offsets are looked up
    /// by name, so a duplicate would silently make one of the two unreachable
    /// — better to fail at build time than to lose a signal at runtime.
    ///
    /// ´claim:signal:two-declarations-of-the-same-name-are-refused-at-build-time-and-the-error-names-the-collision´
    /// ´test:integration:rejects-duplicate-names´
    #[test]
    fn rejects_duplicate_names() {
        let decls = vec![
            scalar_decl("dup", Persistence::Entity),
            scalar_decl("dup", Persistence::Request),
        ];
        let result = SignalSchemaIndex::from_declarations(&decls);
        assert!(matches!(
            result,
            Err(BuildError::DuplicateSignalName { name }) if name == "dup"
        ));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SignalSchemaIndex::encode_all
// ═══════════════════════════════════════════════════════════════════════════════

mod encode_all {
    use super::*;

    /// Encoding places each supplied signal's value at the offset the schema
    /// assigned it, so what the model reads at a column is what the host
    /// supplied for the signal declared there.
    ///
    /// ´claim:signal:encoding-places-each-signal-at-the-offset-the-schema-assigned-it´
    /// ´test:integration:encodes-single-signal´
    #[test]
    fn encodes_single_signal() {
        let decls = vec![scalar_decl("score", Persistence::Entity)];
        let schema = SignalSchemaIndex::from_declarations(&decls).unwrap();

        let mut signals = HashMap::new();
        signals.insert("score".to_string(), SignalValue::Numeric(0.5));

        let features = schema.encode_all(&signals);
        assert_eq!(features, vec![0.5]);
    }

    /// A signal the host did not supply reads as zero in its own column while
    /// the signals around it stay where they were. Absence is encoded in place
    /// rather than by omission, so a sparse request produces a vector of the
    /// same shape as a complete one.
    ///
    /// ´claim:signal:an-unsupplied-signal-reads-as-zero-in-its-own-column-without-shifting-its-neighbours´
    /// ´test:integration:unprovided-signal-is-zero´
    #[test]
    fn unprovided_signal_is_zero() {
        let decls = vec![scalar_decl("a", Persistence::Entity), scalar_decl("b", Persistence::Entity)];
        let schema = SignalSchemaIndex::from_declarations(&decls).unwrap();

        let mut signals = HashMap::new();
        signals.insert("b".to_string(), SignalValue::Numeric(0.75));

        let features = schema.encode_all(&signals);
        assert_eq!(features, vec![0.0, 0.75]);
    }

    /// A name the schema never declared is ignored: the encoded vector keeps
    /// the schema's width and carries only declared signals. A host that sends
    /// more than was agreed cannot widen or reorder the feature space.
    ///
    /// ´claim:signal:an-undeclared-name-is-ignored-rather-than-widening-the-feature-vector´
    /// ´test:integration:ignores-unknown-signals´
    #[test]
    fn ignores_unknown_signals() {
        let decls = vec![scalar_decl("known", Persistence::Entity)];
        let schema = SignalSchemaIndex::from_declarations(&decls).unwrap();

        let mut signals = HashMap::new();
        signals.insert("known".to_string(), SignalValue::Numeric(0.5));
        signals.insert("unknown".to_string(), SignalValue::Numeric(0.9));

        let features = schema.encode_all(&signals);
        assert_eq!(features, vec![0.5]);
    }

    /// A multi-column signal fills every column of its own span: a cyclic
    /// signal declared after a scalar writes its sine into the second position
    /// and its cosine into the third. Wide signals occupy contiguous runs,
    /// which is what makes offsets computed from widths address them
    /// correctly.
    ///
    /// ´claim:signal:a-multi-column-signal-fills-every-column-of-its-own-span´
    /// ´test:integration:encodes-multi-width-signals´
    #[test]
    fn encodes_multi_width_signals() {
        let decls = vec![
            scalar_decl("score", Persistence::Entity),
            SignalDeclaration::new("time", SignalShape::Cyclic { period: 24.0 }, Persistence::Request),
        ];
        let schema = SignalSchemaIndex::from_declarations(&decls).unwrap();

        let mut signals = HashMap::new();
        signals.insert("score".to_string(), SignalValue::Numeric(0.5));
        signals.insert("time".to_string(), SignalValue::Numeric(6.0)); // 6 hours = π/2

        let features = schema.encode_all(&signals);
        assert_eq!(features.len(), 3);
        assert!((features[0] - 0.5).abs() < f64::EPSILON);
        // sin(2π * 6/24) = sin(π/2) = 1
        assert!((features[1] - 1.0).abs() < 1e-10);
        // cos(2π * 6/24) = cos(π/2) = 0
        assert!(features[2].abs() < 1e-10);
    }

    /// A schema with no declarations encodes to an empty vector, so the zero-
    /// signal configuration flows through the encoding path without needing a
    /// special case.
    ///
    /// ´claim:signal:an-empty-schema-encodes-to-an-empty-feature-vector´
    /// ´test:integration:empty-schema-produces-empty-vector´
    #[test]
    fn empty_schema_produces_empty_vector() {
        let schema = SignalSchemaIndex::from_declarations(&[]).unwrap();
        let signals = HashMap::new();
        let features = schema.encode_all(&signals);
        assert_eq!(features, [] as [f64; 0]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SignalSchemaIndex::entity_persistent_mask
// ═══════════════════════════════════════════════════════════════════════════════

mod entity_persistent_mask {
    use super::*;

    /// An empty schema has an empty persistence mask, matching its zero width.
    /// The mask and the vector it describes always have the same length, so
    /// indexing one by the other is safe at every size.
    ///
    /// ´claim:signal:an-empty-schemas-persistence-mask-is-empty-too´
    /// ´test:integration:empty-schema´
    #[test]
    fn empty_schema() {
        let schema = SignalSchemaIndex::from_declarations(&[]).unwrap();
        let mask = schema.entity_persistent_mask();
        assert_eq!(mask, [] as [bool; 0]);
    }

    /// The persistence mask reads true at exactly the positions belonging to
    /// entity-persistent signals. The cache uses it to decide which columns to
    /// remember, so it has to describe positions rather than names.
    ///
    /// ´claim:signal:the-persistence-mask-reads-true-at-exactly-the-entity-persistent-positions´
    /// ´test:integration:all-entity-persistent´
    #[test]
    fn all_entity_persistent() {
        let decls = vec![scalar_decl("a", Persistence::Entity), scalar_decl("b", Persistence::Entity)];
        let schema = SignalSchemaIndex::from_declarations(&decls).unwrap();
        let mask = schema.entity_persistent_mask();
        assert_eq!(mask, vec![true, true]);
    }

    /// A schema of purely request-scoped signals masks to all false, so
    /// nothing in such a configuration is ever written to the entity store.
    ///
    /// (´claim:signal:the-persistence-mask-reads-true-at-exactly-the-entity-persistent-positions´)
    /// ´test:integration:all-request-scoped´
    #[test]
    fn all_request_scoped() {
        let decls = vec![scalar_decl("a", Persistence::Request), scalar_decl("b", Persistence::Request)];
        let schema = SignalSchemaIndex::from_declarations(&decls).unwrap();
        let mask = schema.entity_persistent_mask();
        assert_eq!(mask, vec![false, false]);
    }

    /// Where the two modes are interleaved the mask follows declaration order
    /// position by position, so a request-scoped signal sitting between two
    /// persistent ones is not swept up with them.
    ///
    /// (´claim:signal:the-persistence-mask-reads-true-at-exactly-the-entity-persistent-positions´)
    /// ´test:integration:mixed-persistence´
    #[test]
    fn mixed_persistence() {
        let decls = vec![
            scalar_decl("entity", Persistence::Entity),
            scalar_decl("request", Persistence::Request),
            scalar_decl("entity2", Persistence::Entity),
        ];
        let schema = SignalSchemaIndex::from_declarations(&decls).unwrap();
        let mask = schema.entity_persistent_mask();
        assert_eq!(mask, vec![true, false, true]);
    }

    /// A wide signal's persistence covers every one of its columns: an entity-
    /// persistent cyclic signal marks both its sine and its cosine position.
    /// Remembering half of such a pair would leave the model reading a phase
    /// that never occurred.
    ///
    /// ´claim:signal:a-wide-signals-persistence-covers-every-one-of-its-columns´
    /// ´test:integration:multi-width-signal-mask´
    #[test]
    fn multi_width_signal_mask() {
        let decls = vec![
            scalar_decl("scalar", Persistence::Request),
            SignalDeclaration::new("cyclic", SignalShape::Cyclic { period: 1.0 }, Persistence::Entity),
        ];
        let schema = SignalSchemaIndex::from_declarations(&decls).unwrap();
        let mask = schema.entity_persistent_mask();
        // scalar (width 1, Request) + cyclic (width 2, Entity)
        assert_eq!(mask, vec![false, true, true]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SignalCache operations
// ═══════════════════════════════════════════════════════════════════════════════

mod cache_operations {
    use super::*;

    /// An entity the cache has never seen, asked about with no signals at all,
    /// reads as zeros across the schema's full width. A cold entity yields a
    /// well-formed vector of no evidence rather than a short one or none.
    ///
    /// ´claim:signal:an-unknown-entity-with-no-signals-reads-as-zeros-of-the-full-schema-width´
    /// ´test:integration:miss-returns-zeros-without-signals´
    #[test]
    fn miss_returns_zeros_without_signals() {
        let schema = build_schema(&[scalar_decl("a", Persistence::Entity), scalar_decl("b", Persistence::Request)]);
        let cache = SignalCache::new(100, schema);

        let entity = entity_key("user-1");
        let features = cache.get_and_merge(&entity, &HashMap::new());

        assert_eq!(features, vec![0.0, 0.0]);
    }

    /// An entity-persistent value supplied once comes back on later requests
    /// that do not resupply it. That is what entity persistence is for: an
    /// expensive or occasional signal keeps informing the model between the
    /// requests that can actually produce it.
    ///
    /// ´claim:signal:an-entity-persistent-value-is-returned-on-later-requests-that-do-not-resupply-it´
    /// ´test:integration:hit-returns-cached-entity-persistent´
    #[test]
    fn hit_returns_cached_entity_persistent() {
        let schema = build_schema(&[scalar_decl("auth", Persistence::Entity)]);
        let cache = SignalCache::new(100, schema);

        let entity = entity_key("user-1");

        // First request with auth signal
        let mut signals = HashMap::new();
        signals.insert("auth".to_string(), SignalValue::Numeric(0.9));
        drop(cache.get_and_merge(&entity, &signals));
        // The write is deferred off the merging call; drain to apply it.
        cache.drain_deferred_writes();

        // Second request without auth signal
        let features = cache.get_and_merge(&entity, &HashMap::new());
        assert_eq!(features, vec![0.9]); // Retained from cache
    }

    /// A request-scoped value is visible in the request that supplied it and
    /// gone from the next, which reads zero. Signals describing this request
    /// and not this entity must not be attributed to the entity afterwards.
    ///
    /// ´claim:signal:a-request-scoped-value-lasts-only-for-the-request-that-supplied-it´
    /// ´test:integration:request-scoped-not-cached´
    #[test]
    fn request_scoped_not_cached() {
        let schema = build_schema(&[scalar_decl("ip_rep", Persistence::Request)]);
        let cache = SignalCache::new(100, schema);

        let entity = entity_key("user-1");

        // First request with ip_rep
        let mut signals = HashMap::new();
        signals.insert("ip_rep".to_string(), SignalValue::Numeric(0.8));
        let features1 = cache.get_and_merge(&entity, &signals);
        assert_eq!(features1, vec![0.8]);

        // Second request without ip_rep
        let features2 = cache.get_and_merge(&entity, &HashMap::new());
        assert_eq!(features2, vec![0.0]); // Request-scoped not retained
    }

    /// The merged vector layers this request's signals over the entity's
    /// remembered ones: a request supplying only its own short-lived signal
    /// still reads the persistent one recorded earlier. Neither kind of
    /// evidence has to be resupplied in order to keep the other.
    ///
    /// ´claim:signal:a-merged-vector-layers-this-requests-signals-over-the-entitys-remembered-ones´
    /// ´test:integration:merge-entity-and-request´
    #[test]
    fn merge_entity_and_request() {
        let schema = build_schema(&[
            scalar_decl("auth", Persistence::Entity),
            scalar_decl("ip_rep", Persistence::Request),
        ]);
        let cache = SignalCache::new(100, schema);

        let entity = entity_key("user-1");

        // First request: provide auth
        let mut signals1 = HashMap::new();
        signals1.insert("auth".to_string(), SignalValue::Numeric(0.9));
        drop(cache.get_and_merge(&entity, &signals1));
        cache.drain_deferred_writes();

        // Second request: provide only ip_rep
        let mut signals2 = HashMap::new();
        signals2.insert("ip_rep".to_string(), SignalValue::Numeric(0.5));
        let features = cache.get_and_merge(&entity, &signals2);

        assert_eq!(features, vec![0.9, 0.5]); // auth from cache, ip_rep from request
    }

    /// The cache write is deferred off the merging call: the call that
    /// supplies an entity-persistent value returns it merged, but the cached
    /// copy is untouched until the deferred write is drained — a read before
    /// the drain finds nothing, a read after it finds the value. The merge is
    /// thereby a read plus an enqueue, which is what keeps the assessment
    /// path's write set the three enumerated writes and its additions
    /// enqueues rather than mutations; the drain runs on the identity
    /// maintenance thread's cycle in production.
    ///
    /// ´claim:signal:the-cache-write-is-deferred-off-the-merging-call-and-applied-at-the-drain´
    /// ´test:integration:cache-write-is-deferred-until-the-drain´
    #[test]
    fn cache_write_is_deferred_until_the_drain() {
        let schema = build_schema(&[scalar_decl("auth", Persistence::Entity)]);
        let cache = SignalCache::new(100, schema);
        let entity = entity_key("user-1");

        let mut signals = HashMap::new();
        signals.insert("auth".to_string(), SignalValue::Numeric(0.9));
        let merged = cache.get_and_merge(&entity, &signals);
        assert_eq!(merged, vec![0.9], "the merging call returns the supplied value");

        // Before the drain the cached copy is untouched: the write was an
        // enqueue, not a mutation.
        let before = cache.get_and_merge(&entity, &HashMap::new());
        assert_eq!(before, vec![0.0], "no mutation before the drain");

        cache.drain_deferred_writes();
        let after = cache.get_and_merge(&entity, &HashMap::new());
        assert_eq!(after, vec![0.9], "the drain applies the deferred write");
    }

    /// Resupplying an entity-persistent signal replaces what was remembered
    /// rather than blending with it, so the cache holds the latest reading and
    /// not an average over a history the host never asked it to keep.
    ///
    /// ´claim:signal:resupplying-an-entity-persistent-signal-replaces-the-remembered-value-rather-than-blending-with-it´
    /// ´test:integration:update-overwrites-cached-value´
    #[test]
    fn update_overwrites_cached_value() {
        let schema = build_schema(&[scalar_decl("auth", Persistence::Entity)]);
        let cache = SignalCache::new(100, schema);

        let entity = entity_key("user-1");

        // Initial value
        let mut signals1 = HashMap::new();
        signals1.insert("auth".to_string(), SignalValue::Numeric(0.5));
        drop(cache.get_and_merge(&entity, &signals1));

        // Update value
        let mut signals2 = HashMap::new();
        signals2.insert("auth".to_string(), SignalValue::Numeric(0.9));
        let features = cache.get_and_merge(&entity, &signals2);

        assert_eq!(features, vec![0.9]); // Updated
    }

    /// Each entity's remembered signals are its own: writing one entity's
    /// value leaves another's untouched, and both read back exactly what they
    /// were given. Without that, one entity's history would become evidence
    /// against another.
    ///
    /// ´claim:signal:each-entitys-remembered-signals-are-its-own´
    /// ´test:integration:different-entities-are-independent´
    #[test]
    fn different_entities_are_independent() {
        let schema = build_schema(&[scalar_decl("auth", Persistence::Entity)]);
        let cache = SignalCache::new(100, schema);

        let entity_a = entity_key("user-a");
        let entity_b = entity_key("user-b");

        // Entity A
        let mut signals_a = HashMap::new();
        signals_a.insert("auth".to_string(), SignalValue::Numeric(0.9));
        drop(cache.get_and_merge(&entity_a, &signals_a));

        // Entity B
        let mut signals_b = HashMap::new();
        signals_b.insert("auth".to_string(), SignalValue::Numeric(0.3));
        drop(cache.get_and_merge(&entity_b, &signals_b));
        cache.drain_deferred_writes();

        // Verify independence
        let features_a = cache.get_and_merge(&entity_a, &HashMap::new());
        let features_b = cache.get_and_merge(&entity_b, &HashMap::new());

        assert_eq!(features_a, vec![0.9]);
        assert_eq!(features_b, vec![0.3]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SignalCache eviction
// ═══════════════════════════════════════════════════════════════════════════════

mod eviction {
    use super::*;

    /// A cache at capacity admits a new entity by dropping exactly one
    /// existing one, and counts the drop. The evicted entity is not an error
    /// afterwards — it simply reads as a cold miss again, so a bounded cache
    /// degrades to forgetting rather than to failing.
    ///
    /// ´claim:signal:a-full-cache-admits-a-new-entity-by-dropping-exactly-one-and-counting-it´
    /// ´test:integration:evicts-oldest-at-capacity´
    #[test]
    fn evicts_oldest_at_capacity() {
        let schema = build_schema(&[scalar_decl("auth", Persistence::Entity)]);
        let cache = SignalCache::new(2, schema); // Capacity of 2

        // Insert 3 entities. Writes are deferred and the queue is
        // bounded at the cache's own capacity, so each is drained as
        // it is made.
        for i in 0..3 {
            let entity = entity_key(&format!("user-{i}"));
            let mut signals = HashMap::new();
            signals.insert("auth".to_string(), SignalValue::Numeric(f64::from(i) * 0.1));
            drop(cache.get_and_merge(&entity, &signals));
            cache.drain_deferred_writes();
        }

        // user-0 should be evicted
        let health = cache.health();
        assert_eq!(health.size, 2);
        assert_eq!(
            health.evictions, 1,
            "inserting a 3rd entity at capacity 2 should evict exactly one"
        );

        // user-0 should return zeros (miss)
        let features = cache.get_and_merge(&entity_key("user-0"), &HashMap::new());
        assert_eq!(features, vec![0.0]);
    }

    /// Reading an entity renews it, so eviction falls on the least recently
    /// used rather than the first inserted: an entity read just before the
    /// cache filled survives while an older untouched one goes. Retention
    /// therefore follows live traffic instead of arrival order.
    ///
    /// ´claim:signal:reading-an-entity-renews-it-so-eviction-falls-on-the-least-recently-used-not-the-first-inserted´
    /// ´test:integration:evicts-lru-not-just-oldest´
    #[test]
    fn evicts_lru_not_just_oldest() {
        let schema = build_schema(&[scalar_decl("auth", Persistence::Entity)]);
        let cache = SignalCache::new(3, schema); // Capacity of 3

        // Insert A, B, C
        for name in ["A", "B", "C"] {
            let entity = entity_key(name);
            let mut signals = HashMap::new();
            signals.insert("auth".to_string(), SignalValue::Numeric(1.0));
            drop(cache.get_and_merge(&entity, &signals));
            cache.drain_deferred_writes();
        }

        // Access A to make it recently used (B is now LRU). The recency
        // refresh travels with the deferred write, so drain it too.
        drop(cache.get_and_merge(&entity_key("A"), &HashMap::new()));
        cache.drain_deferred_writes();

        // Insert D - should evict B (oldest untouched), not A
        let mut signals = HashMap::new();
        signals.insert("auth".to_string(), SignalValue::Numeric(1.0));
        drop(cache.get_and_merge(&entity_key("D"), &signals));
        cache.drain_deferred_writes();

        // A should still be cached (was accessed)
        let features_a = cache.get_and_merge(&entity_key("A"), &HashMap::new());
        assert_eq!(features_a, vec![1.0], "A should be retained (recently used)");

        // B should be evicted (LRU)
        let features_b = cache.get_and_merge(&entity_key("B"), &HashMap::new());
        assert_eq!(features_b, vec![0.0], "B should be evicted (LRU)");

        // C and D should still be cached
        let health = cache.health();
        assert_eq!(health.size, 3);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SignalCacheHealth
// ═══════════════════════════════════════════════════════════════════════════════

mod health_tracking {
    use super::*;

    /// Every access is counted as exactly one hit or one miss, the first
    /// access that stores a value included — that one is a miss, because it
    /// found nothing. A hit rate therefore reads against every access the
    /// cache ever served, not against the ones after it warmed.
    ///
    /// ´claim:signal:every-access-counts-as-one-hit-or-one-miss-including-the-first-that-stores-a-value´
    /// ´test:integration:tracks-hits-and-misses´
    #[test]
    fn tracks_hits_and_misses() {
        let schema = build_schema(&[scalar_decl("auth", Persistence::Entity)]);
        let cache = SignalCache::new(100, schema);

        let entity = entity_key("user-1");

        // Store a value (1 miss)
        let mut signals = HashMap::new();
        signals.insert("auth".to_string(), SignalValue::Numeric(0.5));
        drop(cache.get_and_merge(&entity, &signals));
        cache.drain_deferred_writes();

        // 5 hits (same entity)
        for _ in 0..5 {
            drop(cache.get_and_merge(&entity, &HashMap::new()));
        }

        // 5 misses (different entities)
        for i in 0..5 {
            let new_entity = entity_key(&format!("miss-{i}"));
            drop(cache.get_and_merge(&new_entity, &HashMap::new()));
        }

        let health = cache.health();
        assert_eq!(health.hits, 5);
        assert_eq!(health.misses, 6); // 1 initial store + 5 cold lookups
    }

    /// Health reports the configured capacity alongside the number of entities
    /// actually held, and that count tracks insertions. An operator can see
    /// how close a cache is running to its ceiling before evictions start, not
    /// only after.
    ///
    /// ´claim:signal:health-reports-the-configured-capacity-alongside-the-live-entity-count´
    /// ´test:integration:health-reports-correct-capacity-and-size´
    #[test]
    fn health_reports_correct_capacity_and_size() {
        let schema = build_schema(&[scalar_decl("auth", Persistence::Entity)]);
        let cache = SignalCache::new(100, schema);

        let health = cache.health();
        assert_eq!(health.capacity, 100);
        assert_eq!(health.size, 0);

        // Add some entities
        for i in 0..10 {
            let entity = entity_key(&format!("user-{i}"));
            let mut signals = HashMap::new();
            signals.insert("auth".to_string(), SignalValue::Numeric(0.5));
            drop(cache.get_and_merge(&entity, &signals));
        }
        cache.drain_deferred_writes();

        let health = cache.health();
        assert_eq!(health.capacity, 100);
        assert_eq!(health.size, 10);
    }

    /// The hit rate is hits over all accesses, hits and misses together — not
    /// hits against misses, and not against capacity.
    ///
    /// ´claim:signal:the-hit-rate-is-hits-over-all-accesses´
    /// ´test:integration:hit-rate-calculation´
    #[test]
    fn hit_rate_calculation() {
        let health = SignalCacheHealth {
            hits: 75,
            misses: 25,
            evictions: 0,
            capacity: 100,
            size: 50,
        };
        assert!((health.hit_rate() - 0.75).abs() < 1e-10);
    }

    /// A cache that has never been accessed reports a hit rate of zero rather
    /// than a division by nothing, so a freshly started process can be scraped
    /// for metrics before it has served anything.
    ///
    /// ´claim:signal:an-untouched-cache-reports-a-hit-rate-of-zero-rather-than-an-undefined-one´
    /// ´test:integration:hit-rate-zero-when-no-accesses´
    #[test]
    fn hit_rate_zero_when_no_accesses() {
        let health = SignalCacheHealth {
            hits: 0,
            misses: 0,
            evictions: 0,
            capacity: 100,
            size: 0,
        };
        assert!((health.hit_rate() - 0.0).abs() < f64::EPSILON);
    }

    /// Utilisation is the number of entities held over the configured
    /// capacity, so a half-full cache reads as a half regardless of what its
    /// hit and miss counters say.
    ///
    /// ´claim:signal:utilisation-is-the-live-entity-count-over-the-configured-capacity´
    /// ´test:integration:utilisation-calculation´
    #[test]
    fn utilisation_calculation() {
        let health = SignalCacheHealth {
            hits: 0,
            misses: 0,
            evictions: 0,
            capacity: 100,
            size: 50,
        };
        assert!((health.utilisation() - 0.5).abs() < 1e-10);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// NaN handling through cache
// ═══════════════════════════════════════════════════════════════════════════════

mod nan_handling {
    use super::*;

    /// A NaN offered through the cache reaches the merged vector as zero:
    /// sanitisation belongs to encoding, so it guards the caching path as well
    /// as direct encoding.
    ///
    /// (´claim:signal:a-non-finite-scalar-encodes-as-zero-rather-than-entering-the-feature-vector´)
    /// ´test:integration:nan-signal-produces-zero´
    #[test]
    fn nan_signal_produces_zero() {
        let schema = build_schema(&[scalar_decl("score", Persistence::Entity)]);
        let cache = SignalCache::new(100, schema);

        let entity = entity_key("user-1");
        let mut signals = HashMap::new();
        signals.insert("score".to_string(), SignalValue::Numeric(f64::NAN));

        let features = cache.get_and_merge(&entity, &signals);
        assert_eq!(features, vec![0.0]);
    }

    /// What the cache stores is the sanitised encoding and not the raw value,
    /// so an unusable number cannot be read back on a later request either. A
    /// single bad reading is neutralised once rather than remembered and re-
    /// served for the life of the entry.
    ///
    /// ´claim:signal:the-cache-stores-the-sanitised-encoding-so-no-unusable-value-can-be-read-back-later´
    /// ´test:integration:cached-nan-is-sanitised´
    #[test]
    fn cached_nan_is_sanitised() {
        let schema = build_schema(&[scalar_decl("score", Persistence::Entity)]);
        let cache = SignalCache::new(100, schema);

        let entity = entity_key("user-1");

        // Store NaN
        let mut signals = HashMap::new();
        signals.insert("score".to_string(), SignalValue::Numeric(f64::NAN));
        drop(cache.get_and_merge(&entity, &signals));

        // Retrieve without new signals
        let features = cache.get_and_merge(&entity, &HashMap::new());
        assert_eq!(features, vec![0.0]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Edge cases
// ═══════════════════════════════════════════════════════════════════════════════

mod empty_schema {
    use super::*;

    /// A cache over a zero-width schema returns an empty vector rather than a
    /// vector of zeros. The degenerate configuration — nothing declared at all
    /// — travels the ordinary merge path and needs no special case at the call
    /// site.
    ///
    /// ´claim:signal:a-cache-over-a-zero-width-schema-returns-an-empty-vector´
    /// ´test:integration:empty-schema-returns-empty-vector´
    #[test]
    fn empty_schema_returns_empty_vector() {
        let schema = build_schema(&[]);
        let cache = SignalCache::new(100, schema);

        let entity = entity_key("user-1");
        let features = cache.get_and_merge(&entity, &HashMap::new());
        assert_eq!(features, [] as [f64; 0]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Concurrency
// ═══════════════════════════════════════════════════════════════════════════════

mod concurrency {
    use std::thread;

    use super::*;

    /// Four threads merging four hundred distinct entities at once neither
    /// panic nor lose an access: each one is counted as a miss and every
    /// entity ends up held. Concurrency here costs contention on the store's
    /// lock, not correctness of what the store contains.
    ///
    /// ´claim:signal:concurrent-merges-over-distinct-entities-lose-no-access-and-no-entity´
    /// ´test:integration:concurrent-get-merge-no-panics´
    #[test]
    fn concurrent_get_merge_no_panics() {
        let schema = build_schema(&[scalar_decl("auth", Persistence::Entity)]);
        let cache = Arc::new(SignalCache::new(1000, schema));

        let mut handles = Vec::new();

        // Spawn 4 threads, each doing 100 get_and_merge operations
        for thread_id in 0..4 {
            let cache = Arc::clone(&cache);
            let handle = thread::spawn(move || {
                for i in 0..100 {
                    let entity = entity_key(&format!("thread-{thread_id}-entity-{i}"));
                    let mut signals = HashMap::new();
                    signals.insert(
                        "auth".to_string(),
                        SignalValue::Numeric(f64::from(thread_id * 100 + i) * 0.001),
                    );
                    drop(cache.get_and_merge(&entity, &signals));
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
        // Writes were deferred by the merging calls; apply them all.
        cache.drain_deferred_writes();

        // Verify counters sum correctly
        let health = cache.health();
        // All 400 operations were misses (each entity unique)
        assert_eq!(health.misses, 400, "Should have 400 misses (4 threads × 100 ops)");
        assert_eq!(health.hits, 0, "Should have no hits");
        assert_eq!(health.size, 400, "Cache should hold all 400 entities");
    }

    /// When four threads contend on the same ten entities, the hit and miss
    /// counters still sum to exactly the thousand accesses made, the store
    /// settles at one record per entity, and nothing is evicted because
    /// capacity was never approached. How the total splits between hits and
    /// misses is left to the race; the total itself is not.
    ///
    /// ´claim:signal:contended-merges-account-for-every-access-and-keep-one-record-per-entity´
    /// ´test:integration:concurrent-get-merge-with-contention´
    #[test]
    fn concurrent_get_merge_with_contention() {
        let schema = build_schema(&[scalar_decl("auth", Persistence::Entity)]);
        let cache = Arc::new(SignalCache::new(100, schema));

        let mut handles = Vec::new();

        // Spawn 4 threads, all accessing the SAME 10 entities
        for thread_id in 0..4 {
            let cache = Arc::clone(&cache);
            let handle = thread::spawn(move || {
                for round in 0..25 {
                    for entity_id in 0..10 {
                        let entity = entity_key(&format!("shared-{entity_id}"));
                        let mut signals = HashMap::new();
                        signals.insert(
                            "auth".to_string(),
                            SignalValue::Numeric(f64::from(thread_id * 1000 + round * 10 + entity_id) * 0.001),
                        );
                        drop(cache.get_and_merge(&entity, &signals));
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        // Verify no panics occurred and counters are reasonable
        // Writes were deferred by the merging calls; apply them all.
        cache.drain_deferred_writes();

        let health = cache.health();
        // Total operations: 4 threads × 25 rounds × 10 entities = 1000
        // First 10 are misses, rest are hits (but due to race conditions, counts may vary)
        assert_eq!(health.hits + health.misses, 1000, "Total accesses should be 1000");
        assert_eq!(health.size, 10, "Cache should hold exactly 10 entities");
        assert_eq!(health.evictions, 0, "No evictions needed (capacity 100 > 10 entities)");
    }
}
