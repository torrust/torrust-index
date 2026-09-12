## Unit test matrix · `tab:assayer:linalg-unit-test-matrix`

**Table (Unit test matrix)**

| Test | Area | Claim |
|------|------|-------|
| (`test:unit:matrix-dimension-overflow-is-a-deserialisation-error`) | persistence | Matrix dimension multiplication reports malformed persisted input on overflow, in every build profile, rather than panicking before the payload length can be rejected. |
| (`test:unit:symmetric-dimension-overflow-is-a-deserialisation-error`) | persistence | Squaring a symmetric matrix dimension reports malformed persisted input on overflow rather than wrapping to the empty payload's length. |
