// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use crate::Pewei;
use crate::pewei::{Layer, Terminal, Transition};

// ── Transition::snr ─────────────────────────────────────────

#[test]
fn snr_nonzero_baseline() {
    let t = Transition::<u64, u64> {
        start: 0,
        end: 128,
        baseline: 10,
        total: 30,
        refinement: 20,
        depth: 1,
        v_depth: 0,
    };
    let snr = t.snr().expect("baseline is nonzero");
    assert!((snr - 2.0).abs() < f64::EPSILON);
}

#[test]
fn snr_zero_baseline_returns_none() {
    let t = Transition::<u64, u64> {
        start: 0,
        end: 128,
        baseline: 0,
        total: 5,
        refinement: 5,
        depth: 1,
        v_depth: 0,
    };
    assert!(t.snr().is_none());
}

#[test]
fn snr_float_types() {
    let t = Transition::<f64, f64> {
        start: 0.0,
        end: 8.0,
        baseline: 4.0,
        total: 10.0,
        refinement: 6.0,
        depth: 1,
        v_depth: 0,
    };
    let snr = t.snr().expect("baseline is nonzero");
    assert!((snr - 1.5).abs() < f64::EPSILON);
}

// ── Width ───────────────────────────────────────────────────

#[test]
fn transition_width() {
    let t = Transition::<u64, u64> {
        start: 64,
        end: 192,
        baseline: 0,
        total: 0,
        refinement: 0,
        depth: 1,
        v_depth: 0,
    };
    assert_eq!(t.width(), 128);
}

#[test]
fn terminal_width() {
    let t = Terminal::<u64, u64> {
        start: 0,
        end: 256,
        intensity: 42,
        depth: 0,
        v_depth: 0,
    };
    assert_eq!(t.width(), 256);
}

// ── Pewei convenience methods ───────────────────────────────

#[test]
fn empty_pewei_counts() {
    let p = Pewei::<u64, u64> {
        domain_start: 0,
        domain_end: 256,
        layers: vec![],
    };
    assert_eq!(p.layer_count(), 0);
    assert_eq!(p.node_count(), 0);
    assert_eq!(p.total_energy(), 0);
}

#[test]
fn pewei_node_count_and_energy() {
    let p = Pewei::<u64, u64> {
        domain_start: 0,
        domain_end: 256,
        layers: vec![
            Layer {
                transitions: vec![Transition {
                    start: 0,
                    end: 256,
                    baseline: 10,
                    total: 50,
                    refinement: 40,
                    depth: 0,
                    v_depth: 0,
                }],
                terminals: vec![],
            },
            Layer {
                transitions: vec![],
                terminals: vec![
                    Terminal {
                        start: 0,
                        end: 128,
                        intensity: 25,
                        depth: 1,
                        v_depth: 1,
                    },
                    Terminal {
                        start: 128,
                        end: 256,
                        intensity: 15,
                        depth: 1,
                        v_depth: 1,
                    },
                ],
            },
        ],
    };
    assert_eq!(p.layer_count(), 2);
    assert_eq!(p.node_count(), 3);
    // total_energy = transition.baseline + terminal intensities
    // = 10 + 25 + 15 = 50  (equals the G-root sum)
    assert_eq!(p.total_energy(), 50);
}

// ── Serde round-trip ────────────────────────────────────────

#[cfg(feature = "serde")]
#[test]
fn serde_round_trip_json() {
    let original = Pewei::<u64, u64> {
        domain_start: 0,
        domain_end: 256,
        layers: vec![Layer {
            transitions: vec![],
            terminals: vec![Terminal {
                start: 0,
                end: 256,
                intensity: 42,
                depth: 0,
                v_depth: 0,
            }],
        }],
    };
    let json = serde_json::to_string(&original).expect("serialize");
    let restored: Pewei<u64, u64> = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, restored);
}

#[cfg(feature = "serde")]
#[test]
fn serde_round_trip_transition() {
    let t = Transition::<u64, u64> {
        start: 0,
        end: 128,
        baseline: 10,
        total: 30,
        refinement: 20,
        depth: 1,
        v_depth: 0,
    };
    let json = serde_json::to_string(&t).expect("serialize");
    let restored: Transition<u64, u64> = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(t, restored);
}

// ── Reconstruction ──────────────────────────────────────────

/// Helper: sum all span intensities.
fn span_total(spans: &[crate::view::Span<u64, u64>]) -> u64 {
    spans.iter().map(|s| s.intensity).sum()
}

#[test]
fn reconstruct_empty_pewei() {
    let p = Pewei::<u64, u64> {
        domain_start: 0,
        domain_end: 256,
        layers: vec![],
    };
    let spans = p.reconstruct(0);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].start, 0);
    assert_eq!(spans[0].end, 256);
    assert_eq!(spans[0].intensity, 0);
}

#[test]
fn reconstruct_single_terminal() {
    // One terminal at root depth — the simplest non-empty PEWEI.
    let p = Pewei::<u64, u64> {
        domain_start: 0,
        domain_end: 256,
        layers: vec![Layer {
            transitions: vec![],
            terminals: vec![Terminal {
                start: 0,
                end: 256,
                intensity: 42,
                depth: 0,
                v_depth: 0,
            }],
        }],
    };
    let spans = p.reconstruct(0);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].start, 0);
    assert_eq!(spans[0].end, 256);
    assert_eq!(spans[0].intensity, 42);
}

#[test]
fn reconstruct_single_transition_truncated() {
    // One transition, max_layer=0 → fully truncated → emit total.
    let p = Pewei::<u64, u64> {
        domain_start: 0,
        domain_end: 256,
        layers: vec![
            Layer {
                transitions: vec![Transition {
                    start: 0,
                    end: 256,
                    baseline: 10,
                    total: 50,
                    refinement: 40,
                    depth: 0,
                    v_depth: 0,
                }],
                terminals: vec![],
            },
            Layer {
                transitions: vec![],
                terminals: vec![
                    Terminal {
                        start: 0,
                        end: 128,
                        intensity: 25,
                        depth: 1,
                        v_depth: 1,
                    },
                    Terminal {
                        start: 128,
                        end: 256,
                        intensity: 15,
                        depth: 1,
                        v_depth: 1,
                    },
                ],
            },
        ],
    };
    // Truncate at layer 0 — children not visible.
    let spans = p.reconstruct(0);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].intensity, 50); // accumulated(0) + total(50)
}

#[test]
fn reconstruct_full_depth_matches_terminals() {
    // Root transition (baseline=10, total=50) with two terminal
    // children (25 and 15). Full reconstruction should produce
    // two spans whose intensities include the pro-rated baseline.
    //
    // half_bg = (0 + 10).prorate(1, 2) = 5
    // left:  5 + 25 = 30
    // right: 5 + 15 = 20
    // total: 30 + 20 = 50 ✓
    let p = Pewei::<u64, u64> {
        domain_start: 0,
        domain_end: 256,
        layers: vec![
            Layer {
                transitions: vec![Transition {
                    start: 0,
                    end: 256,
                    baseline: 10,
                    total: 50,
                    refinement: 40,
                    depth: 0,
                    v_depth: 0,
                }],
                terminals: vec![],
            },
            Layer {
                transitions: vec![],
                terminals: vec![
                    Terminal {
                        start: 0,
                        end: 128,
                        intensity: 25,
                        depth: 1,
                        v_depth: 1,
                    },
                    Terminal {
                        start: 128,
                        end: 256,
                        intensity: 15,
                        depth: 1,
                        v_depth: 1,
                    },
                ],
            },
        ],
    };
    let spans = p.reconstruct(1);
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0].start, 0);
    assert_eq!(spans[0].end, 128);
    assert_eq!(spans[0].intensity, 30); // half_bg(5) + 25
    assert_eq!(spans[1].start, 128);
    assert_eq!(spans[1].end, 256);
    assert_eq!(spans[1].intensity, 20); // half_bg(5) + 15
}

#[test]
fn reconstruct_energy_conservation_at_each_layer() {
    // 3-layer PEWEI:
    //   Layer 0: root transition [0,256) baseline=20, total=100
    //   Layer 1: left transition [0,128) baseline=10, total=60
    //            right terminal  [128,256) intensity=20
    //   Layer 2: left-left terminal  [0,64)  intensity=30
    //            left-right terminal [64,128) intensity=20
    //
    // Note: refinement for root  = 100 - 20 = 80
    //       refinement for left  = 60 - 10 = 50
    //       Root's children totals: 60 + 20 = 80 = refinement ✓
    //       Left's children totals: 30 + 20 = 50 = refinement ✓
    let p = Pewei::<u64, u64> {
        domain_start: 0,
        domain_end: 256,
        layers: vec![
            Layer {
                transitions: vec![Transition {
                    start: 0,
                    end: 256,
                    baseline: 20,
                    total: 100,
                    refinement: 80,
                    depth: 0,
                    v_depth: 0,
                }],
                terminals: vec![],
            },
            Layer {
                transitions: vec![Transition {
                    start: 0,
                    end: 128,
                    baseline: 10,
                    total: 60,
                    refinement: 50,
                    depth: 1,
                    v_depth: 1,
                }],
                terminals: vec![Terminal {
                    start: 128,
                    end: 256,
                    intensity: 20,
                    depth: 1,
                    v_depth: 1,
                }],
            },
            Layer {
                transitions: vec![],
                terminals: vec![
                    Terminal {
                        start: 0,
                        end: 64,
                        intensity: 30,
                        depth: 2,
                        v_depth: 2,
                    },
                    Terminal {
                        start: 64,
                        end: 128,
                        intensity: 20,
                        depth: 2,
                        v_depth: 2,
                    },
                ],
            },
        ],
    };

    // At every truncation level, total energy must be 100.
    assert_eq!(span_total(&p.reconstruct(0)), 100);
    assert_eq!(span_total(&p.reconstruct(1)), 100);
    assert_eq!(span_total(&p.reconstruct(2)), 100);
}

#[test]
fn reconstruct_one_visible_one_invisible_remainder() {
    // Root transition [0,256): baseline=10, total=50, refinement=40.
    // Only left child visible: terminal [0,128) intensity=25.
    // Right child invisible — remainder = 40 - 25 = 15.
    //
    // half_bg = (0 + 10).prorate(1,2) = 5
    // left span:  5 + 25 = 30
    // right span: 5 + 15 = 20
    // total: 30 + 20 = 50 ✓
    let p = Pewei::<u64, u64> {
        domain_start: 0,
        domain_end: 256,
        layers: vec![
            Layer {
                transitions: vec![Transition {
                    start: 0,
                    end: 256,
                    baseline: 10,
                    total: 50,
                    refinement: 40,
                    depth: 0,
                    v_depth: 0,
                }],
                terminals: vec![],
            },
            Layer {
                transitions: vec![],
                terminals: vec![
                    Terminal {
                        start: 0,
                        end: 128,
                        intensity: 25,
                        depth: 1,
                        v_depth: 1,
                    },
                    // Right child NOT present — invisible.
                ],
            },
        ],
    };
    let spans = p.reconstruct(1);
    assert_eq!(spans.len(), 2);
    // Left: visible terminal.
    assert_eq!(spans[0].start, 0);
    assert_eq!(spans[0].end, 128);
    assert_eq!(spans[0].intensity, 30); // half_bg(5) + 25
    // Right: invisible sibling remainder.
    assert_eq!(spans[1].start, 128);
    assert_eq!(spans[1].end, 256);
    assert_eq!(spans[1].intensity, 20); // half_bg(5) + remainder(15)
    assert_eq!(span_total(&spans), 50);
}

#[test]
fn reconstruct_right_visible_left_invisible() {
    // Same as above but mirrored — only right child visible.
    let p = Pewei::<u64, u64> {
        domain_start: 0,
        domain_end: 256,
        layers: vec![
            Layer {
                transitions: vec![Transition {
                    start: 0,
                    end: 256,
                    baseline: 10,
                    total: 50,
                    refinement: 40,
                    depth: 0,
                    v_depth: 0,
                }],
                terminals: vec![],
            },
            Layer {
                transitions: vec![],
                terminals: vec![Terminal {
                    start: 128,
                    end: 256,
                    intensity: 15,
                    depth: 1,
                    v_depth: 1,
                }],
            },
        ],
    };
    let spans = p.reconstruct(1);
    assert_eq!(spans.len(), 2);
    // Left: invisible sibling. remainder = 40 - 15 = 25.
    assert_eq!(spans[0].start, 0);
    assert_eq!(spans[0].end, 128);
    assert_eq!(spans[0].intensity, 30); // half_bg(5) + 25
    // Right: visible terminal.
    assert_eq!(spans[1].start, 128);
    assert_eq!(spans[1].end, 256);
    assert_eq!(spans[1].intensity, 20); // half_bg(5) + 15
    assert_eq!(span_total(&spans), 50);
}

#[test]
fn reconstruct_both_children_visible() {
    // Symmetric case — both children terminals.
    let p = Pewei::<u64, u64> {
        domain_start: 0,
        domain_end: 256,
        layers: vec![
            Layer {
                transitions: vec![Transition {
                    start: 0,
                    end: 256,
                    baseline: 10,
                    total: 50,
                    refinement: 40,
                    depth: 0,
                    v_depth: 0,
                }],
                terminals: vec![],
            },
            Layer {
                transitions: vec![],
                terminals: vec![
                    Terminal {
                        start: 0,
                        end: 128,
                        intensity: 25,
                        depth: 1,
                        v_depth: 1,
                    },
                    Terminal {
                        start: 128,
                        end: 256,
                        intensity: 15,
                        depth: 1,
                        v_depth: 1,
                    },
                ],
            },
        ],
    };
    let spans = p.reconstruct(1);
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0].intensity, 30); // 5 + 25
    assert_eq!(spans[1].intensity, 20); // 5 + 15
}

#[test]
fn reconstruct_no_gaps_no_overlaps() {
    // Verify output partitions the domain.
    let p = Pewei::<u64, u64> {
        domain_start: 0,
        domain_end: 256,
        layers: vec![
            Layer {
                transitions: vec![Transition {
                    start: 0,
                    end: 256,
                    baseline: 20,
                    total: 100,
                    refinement: 80,
                    depth: 0,
                    v_depth: 0,
                }],
                terminals: vec![],
            },
            Layer {
                transitions: vec![Transition {
                    start: 0,
                    end: 128,
                    baseline: 10,
                    total: 60,
                    refinement: 50,
                    depth: 1,
                    v_depth: 1,
                }],
                terminals: vec![Terminal {
                    start: 128,
                    end: 256,
                    intensity: 20,
                    depth: 1,
                    v_depth: 1,
                }],
            },
            Layer {
                transitions: vec![],
                terminals: vec![
                    Terminal {
                        start: 0,
                        end: 64,
                        intensity: 30,
                        depth: 2,
                        v_depth: 2,
                    },
                    Terminal {
                        start: 64,
                        end: 128,
                        intensity: 20,
                        depth: 2,
                        v_depth: 2,
                    },
                ],
            },
        ],
    };

    for max_layer in 0..=2 {
        let spans = p.reconstruct(max_layer);
        assert!(!spans.is_empty(), "layer {max_layer}: no spans");

        // First span starts at domain_start.
        assert_eq!(spans[0].start, 0, "layer {max_layer}: start");

        // Last span ends at domain_end.
        assert_eq!(spans.last().unwrap().end, 256, "layer {max_layer}: end");

        // Each span's end == next span's start.
        for w in spans.windows(2) {
            assert_eq!(
                w[0].end, w[1].start,
                "layer {max_layer}: gap between {} and {}",
                w[0].end, w[1].start
            );
        }
    }
}

#[test]
fn reconstruct_integer_rounding() {
    // Odd baseline: 11.prorate(1, 2) = 5 (truncation).
    // Each half gets 5 of background, losing 1 unit total.
    let p = Pewei::<u64, u64> {
        domain_start: 0,
        domain_end: 256,
        layers: vec![
            Layer {
                transitions: vec![Transition {
                    start: 0,
                    end: 256,
                    baseline: 11,
                    total: 51,
                    refinement: 40,
                    depth: 0,
                    v_depth: 0,
                }],
                terminals: vec![],
            },
            Layer {
                transitions: vec![],
                terminals: vec![
                    Terminal {
                        start: 0,
                        end: 128,
                        intensity: 25,
                        depth: 1,
                        v_depth: 1,
                    },
                    Terminal {
                        start: 128,
                        end: 256,
                        intensity: 15,
                        depth: 1,
                        v_depth: 1,
                    },
                ],
            },
        ],
    };
    let spans = p.reconstruct(1);
    // half_bg = 11.prorate(1, 2) = 5
    assert_eq!(spans[0].intensity, 30); // 5 + 25
    assert_eq!(spans[1].intensity, 20); // 5 + 15
    // Total = 50, original was 51. Lost 1 unit to rounding.
    assert_eq!(span_total(&spans), 50);
}

#[test]
fn reconstruct_f64_coordinates() {
    let p = Pewei::<f64, f64> {
        domain_start: 0.0,
        domain_end: 8.0,
        layers: vec![
            Layer {
                transitions: vec![Transition {
                    start: 0.0,
                    end: 8.0,
                    baseline: 2.0,
                    total: 10.0,
                    refinement: 8.0,
                    depth: 0,
                    v_depth: 0,
                }],
                terminals: vec![],
            },
            Layer {
                transitions: vec![],
                terminals: vec![
                    Terminal {
                        start: 0.0,
                        end: 4.0,
                        intensity: 5.0,
                        depth: 1,
                        v_depth: 1,
                    },
                    Terminal {
                        start: 4.0,
                        end: 8.0,
                        intensity: 3.0,
                        depth: 1,
                        v_depth: 1,
                    },
                ],
            },
        ],
    };
    let spans = p.reconstruct(1);
    assert_eq!(spans.len(), 2);
    // half_bg = (0 + 2).prorate(1, 2) = 1.0
    assert!((spans[0].intensity - 6.0).abs() < f64::EPSILON); // 1.0 + 5.0
    assert!((spans[1].intensity - 4.0).abs() < f64::EPSILON); // 1.0 + 3.0
    let total: f64 = spans.iter().map(|s| s.intensity).sum();
    assert!((total - 10.0).abs() < f64::EPSILON);
}

#[test]
fn reconstruct_max_layer_clamped() {
    // max_layer beyond layer_count should behave like full depth.
    let p = Pewei::<u64, u64> {
        domain_start: 0,
        domain_end: 256,
        layers: vec![Layer {
            transitions: vec![],
            terminals: vec![Terminal {
                start: 0,
                end: 256,
                intensity: 42,
                depth: 0,
                v_depth: 0,
            }],
        }],
    };
    let a = p.reconstruct(0);
    let b = p.reconstruct(100);
    assert_eq!(a, b);
}

#[test]
fn reconstruct_output_sorted_by_start() {
    let p = Pewei::<u64, u64> {
        domain_start: 0,
        domain_end: 256,
        layers: vec![
            Layer {
                transitions: vec![Transition {
                    start: 0,
                    end: 256,
                    baseline: 20,
                    total: 100,
                    refinement: 80,
                    depth: 0,
                    v_depth: 0,
                }],
                terminals: vec![],
            },
            Layer {
                transitions: vec![Transition {
                    start: 0,
                    end: 128,
                    baseline: 10,
                    total: 60,
                    refinement: 50,
                    depth: 1,
                    v_depth: 1,
                }],
                terminals: vec![Terminal {
                    start: 128,
                    end: 256,
                    intensity: 20,
                    depth: 1,
                    v_depth: 1,
                }],
            },
            Layer {
                transitions: vec![],
                terminals: vec![
                    Terminal {
                        start: 0,
                        end: 64,
                        intensity: 30,
                        depth: 2,
                        v_depth: 2,
                    },
                    Terminal {
                        start: 64,
                        end: 128,
                        intensity: 20,
                        depth: 2,
                        v_depth: 2,
                    },
                ],
            },
        ],
    };
    for max_layer in 0..=2 {
        let spans = p.reconstruct(max_layer);
        for w in spans.windows(2) {
            assert!(
                w[0].start < w[1].start,
                "layer {max_layer}: not sorted — {} >= {}",
                w[0].start,
                w[1].start
            );
        }
    }
}
