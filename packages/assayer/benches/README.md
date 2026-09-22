## Bench test matrix · `tab:assayer:bench-test-matrix`

**Table (Bench test matrix)**

| Test | Area | Claim |
|------|------|-------|
| (`test:bench:bench-assessment`) | audit | Core assessment and host derivation are measured separately over cold and reference-width worlds, preserving the former operating-cycle witness without concealing the read/write split; the documented comparison floor is six milliseconds per completed assess–derive–label round trip, and the baseline is compared rather than asserted. |
| (`test:bench:bench-label-publication`) | audit | Complete real-engine label publication and its sequential dense model-update region are measured separately on the representative workload, preserving the former attribution witness on one publication path; the attribution split has no asserted floor, while the documented cross-group floor is six milliseconds per completed round trip and is compared rather than asserted. |
| (`test:bench:bench-construction-shutdown`) | audit | Building and dropping a guarded real-engine world is measured across every declared performance configuration, including the owner-thread join; the documented comparison floor is three hundred milliseconds per complete build/drop cycle, and the baseline is compared rather than asserted. |
| (`test:bench:bench-pre-seed`) | audit | One synchronous pre-seed call is measured over the guarded thousand-entry population and returns only after every entry has been processed; the documented comparison floor is one thousand accepted entries within thirty seconds, and the baseline is compared rather than asserted. |
| (`test:bench:bench-health-summary`) | audit | Health-summary polling is measured without mutation over cold and populated worlds; the documented comparison floor is one millisecond per call, and the baseline is compared rather than asserted. |
| (`test:bench:bench-full-health-report`) | audit | Full-health-report assembly is measured separately from its summary over cold and populated worlds so its per-Sentinel, per-dimension, and per-model walk remains visible; the documented comparison floor is fifty milliseconds per call, and the baseline is compared rather than asserted. |
| (`test:bench:bench-reference-marginalisation`) | bayes | Deregistering one Sentinel is measured on a guarded, trained reference-width world through the real lifecycle barrier, replacing the private-model Schur witness; the documented comparison floor is three hundred milliseconds per completed removal, and the baseline is compared rather than asserted. |
