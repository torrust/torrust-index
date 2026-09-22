## Unit test matrix · `tab:assayer:testing-unit-test-matrix`

**Table (Unit test matrix)**

| Test | Area | Claim |
|------|------|-------|
| (`test:unit:the-exact-tolerance-admits-the-two-zeros`) | harness | The exact tolerance is numeric equality: it reads the two zeros as one value, which is the whole difference between a bound and an identity. |
| (`test:unit:the-bit-comparison-separates-the-two-zeros`) | harness | cites (`claim:harness:a-zero-tolerance-is-numeric-equality-so-a-bit-identity-promise-is-checked-by-the-bits`) |
| (`test:unit:the-bit-comparison-holds-a-value-against-itself`) | harness | cites (`claim:harness:a-zero-tolerance-is-numeric-equality-so-a-bit-identity-promise-is-checked-by-the-bits`) |
| (`test:unit:held-barrier-completes-before-its-checkpoint`) | harness | A held barrier completes before its row checkpoint, so a settled comparison cannot run early. |
| (`test:unit:skipped-checkpoint-is-rejected`) | harness | cites (`claim:harness:a-declared-playback-checkpoint-runs-exactly-once-at-its-row-boundary`) |
| (`test:unit:duplicated-checkpoint-is-rejected`) | harness | cites (`claim:harness:a-declared-playback-checkpoint-runs-exactly-once-at-its-row-boundary`) |
| (`test:unit:early-checkpoint-is-rejected`) | harness | cites (`claim:harness:a-declared-playback-checkpoint-runs-exactly-once-at-its-row-boundary`) |
| (`test:unit:later-boundary-checkpoint-is-rejected`) | harness | An invocation after the declared row is rejected as a later boundary. |
| (`test:unit:stalled-progress-names-the-first-incomplete-row`) | harness | Progress advances once for a completed row and names the next row while that row is deliberately blocked. |
| (`test:unit:batch-size-preserves-mid-batch-publication-boundaries`) | harness | A row after a mid-batch state change sees the preceding row's publication for every bounded batch size. |
| (`test:unit:a-missing-scan-root-fails-the-audit-naming-the-path`) | audit | A scan root that cannot be read is a failed audit, not an empty tree. |
| (`test:unit:an-unreadable-source-file-fails-the-audit-naming-the-path`) | audit | cites (`claim:audit:a-source-audit-that-cannot-read-the-tree-fails-instead-of-reporting-it-clean`) |
| (`test:unit:a-guarded-variant-does-not-hide-the-items-after-it`) | audit | A guarded enum variant covers the variant and stops there. |
| (`test:unit:a-guarded-declaration-does-not-hide-the-rest-of-the-file`) | audit | cites (`claim:audit:a-cfg-test-attribute-covers-the-item-it-guards-and-the-lines-after-it-stay-production`) |
| (`test:unit:a-guarded-module-does-not-hide-the-items-after-it`) | audit | cites (`claim:audit:a-cfg-test-attribute-covers-the-item-it-guards-and-the-lines-after-it-stay-production`) |
| (`test:unit:nested-braces-inside-a-guarded-item-do-not-end-it-early`) | audit | cites (`claim:audit:a-cfg-test-attribute-covers-the-item-it-guards-and-the-lines-after-it-stay-production`) |
| (`test:unit:a-guarded-variant-no-longer-hides-the-maintenance-loop`) | audit | cites (`claim:audit:a-cfg-test-attribute-covers-the-item-it-guards-and-the-lines-after-it-stay-production`) |
