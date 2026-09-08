# ADR-S-001: Measures Not Opinions · `rec:sentinel:raw-measurements-with-host-owned-policy`

**Status:** Implemented **Date:** 2026-03-08 **Spec:** §ALGO S-1.2 (layer responsibilities — "sentinel measures; host decides")

## Context · `sec:sentinel:measure-context`

An anomaly detector can either:

- **A)** Output raw statistical measurements and let the consumer decide what they mean (library approach).
- **B)** Output verdicts — threat levels, recommended actions, block/allow decisions (appliance approach).

The sentinel is a library embedded inside the Torrust Index. Different hosts have different risk tolerances, different action vocabularies (ban, throttle, flag, ignore), and different false-positive consequences. Baking policy into the sentinel would force every host into one policy model.

## Decision · `sec:sentinel:measure-decision`

**The sentinel outputs only raw statistical measurements. It never outputs opinions, threat levels, or recommended actions.**

Concretely:

- `BatchReport` contains `AnomalyScores` (per-axis mean, max, z-score, CUSUM), `ScoringGeometry` (ADR-S-008), `TrackerMaturity`, rank, energy ratios.
- No field is named "threat", "risk", "anomaly_level", or "action".
- No method returns a boolean "is anomalous" verdict.
- No internal threshold triggers automatic remediation.

The host reads the report and applies its own policy:

```rust
// Host policy — not sentinel code:
if report.scores.novelty.z_score > 4.0 && report.maturity.noise_influence < 0.1 {
    throttle(cell_id);
}
```

## Consequences · `sec:sentinel:measure-consequences`

- The sentinel has no policy parameters (no "alert threshold", no "sensitivity level").
- Report types carry more fields than an appliance would expose, but each field has a precise statistical definition.
- Integration tests assert statistical properties, not verdicts.
- Higher polarity = more anomalous is a uniform convention across all four scoring axes (§ALGO S-6), ensuring the host can apply a single threshold logic to any axis.
