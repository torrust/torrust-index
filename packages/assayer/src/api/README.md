## Unit test matrix · `tab:assayer:api-unit-test-matrix`

**Table (Unit test matrix)**

| Test | Area | Claim |
|------|------|-------|
| (`test:unit:sanitise-nan-valence`) | labelling | A valence that is not a number is repaired to zero at the boundary rather than refused, and the repair is counted: the sanitisation record beside the payload reports one valence repaired. The label still reaches the model owner, carrying no signal instead of poisoning every scalar downstream — and because zero is also an ordinary valence, the count is the only trace the repair leaves (´dec:surface:sanitise-not-reject´). |
| (`test:unit:sanitise-inf-valence`) | labelling | cites (`claim:labelling:a-non-finite-valence-is-repaired-to-zero-and-the-repair-is-counted`) |
| (`test:unit:sanitise-nan-outcome`) | labelling | Per-axis outcomes are repaired individually and the drops are counted: the axis whose value is not finite leaves the map, its finite sibling stays, and the sanitisation record reports exactly one outcome dropped. One broken axis therefore costs the host that axis only — and visibly, since a dropped axis leaves no other trace on the label (´dec:surface:sanitise-not-reject´). |
| (`test:unit:sanitise-normal-values-untouched`) | labelling | Repair is confined to what is actually broken: an ordinary valence and an ordinary outcome come out of the boundary bit-for-bit as they went in, and the sanitisation record reports nothing repaired. The sanitiser is a filter on non-finite values, not a transform every label pays for — and a count that rose on clean labels would make the diagnostic it feeds meaningless (´dec:surface:sanitise-not-reject´). |
| (`test:unit:mudlark-config-uses-registered-identity-thresholds`) | identity | Every graph threshold in the Mudlark configuration is copied from the public identity budget, while the shipped budget constructor reproduces the former derived values for the registration's competitive cutoff. |
