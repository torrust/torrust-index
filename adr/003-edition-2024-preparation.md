# ADR-T-003: Preparing for Rust Edition 2024

**Status:** Decided
**Date:** 2026-03-23
**Relates to:** [ADR-T-001](./001-lowercase-infohashes.md)
(root crate conventions)

## Context

Rust edition 2024 (stabilised in Rust 1.85) introduces language and
library changes that affect compilation. Although the workspace still
targets `edition = "2021"`, several transitive dependencies have
already published releases that set `edition = "2024"` in their own
`Cargo.toml`. Those crates require `rustc >= 1.85` to compile.

This creates a conflict: keeping the workspace on edition 2021 is
desirable until all first-party code is migrated, but the latest
versions of some dependencies can no longer build on the previous
MSRV (1.80).

Even after downgrading every edition-2024 crate, the remaining
dependency tree still requires at least **rustc 1.83** due to
non-edition MSRV bumps in transitive crates:

| Minimum rustc | Crates (examples)                                                          |
| ------------- | -------------------------------------------------------------------------- |
| 1.83          | `async-compression`, `compression-codecs`, `crc`, `icu_*`, `pest*`         |
| 1.82          | `axum-server`, `idna_adapter`, `indexmap`, `serde_with`, `yoke`, `zerovec` |
| 1.81          | `base64ct`, `derive_more`, `zerofrom`                                      |

Since 1.83 is the highest floor in that set, it becomes the new MSRV.

## Options Considered

| Option                           | Strategy                                                                               | Edition compat                    | Risk                                                                            |
| -------------------------------- | -------------------------------------------------------------------------------------- | --------------------------------- | ------------------------------------------------------------------------------- |
| A. Pin deps, raise MSRV to 1.83  | Downgrade or pin every transitive dep that requires edition 2024; stay on edition 2021 | All deps build under 2021         | Pins will drift; must be actively maintained until the full migration           |
| B. Jump straight to edition 2024 | Bump `edition = "2024"` workspace-wide, require rustc >= 1.85                          | Native                            | Large migration surface — all first-party code must pass 2024 lints in one step |
| C. Stay on current MSRV (1.80)   | Pin even more aggressively, avoid the 1.81–1.83 crates too                             | All deps build under 2021 on 1.80 | Too many pins; some crates have no viable older version                         |

Option C was rejected because the transitive dependency floor at 1.83
cannot be avoided without forking crates. Option B is the end goal
but premature — edition 2024 migration should be a separate,
focused ADR. Option A buys time for that migration.

## Decision

**Option A — Pin or downgrade affected transitive dependencies so
the entire dependency tree compiles under edition 2021 with a raised
MSRV of 1.83.**

### Direct dependency changes

| Setting / Crate | From                    | To   | Reason                                                        |
| --------------- | ----------------------- | ---- | ------------------------------------------------------------- |
| `rust-version`  | 1.80                    | 1.83 | Highest MSRV required by transitive deps                      |
| `jsonwebtoken`  | 10 (with `rust_crypto`) | 9.3  | v10 pulls `simple_asn1 0.6.4` -> `time 0.3.47` (edition 2024) |
| `rand`          | 0 (resolved to 0.10)    | 0.9  | `rand 0.10` depends on `rand_core 0.10` (edition 2024)        |

### Cargo.lock downgrades

The following crates were pinned to older versions to avoid
edition-2024-only releases in the lock file:

| Crate               | From    | To      | Chain                                      |
| ------------------- | ------- | ------- | ------------------------------------------ |
| `time`              | 0.3.47  | 0.3.41  | `time-core 0.1.8` -> edition 2024          |
| `time-core`         | 0.1.8   | 0.1.4   | edition 2024                               |
| `time-macros`       | 0.2.27  | 0.2.22  | pulled by `time`                           |
| `deranged`          | 0.5.8   | 0.4.0   | pulled by `time`                           |
| `num-conv`          | 0.2.0   | 0.1.0   | pulled by `time`                           |
| `simple_asn1`       | 0.6.4   | 0.6.3   | required `time ^0.3.47`                    |
| `serde_with`        | 3.18.0  | 3.17.0  | required `time ~0.3.47`                    |
| `serde_with_macros` | 3.18.0  | 3.17.0  | pulled by `serde_with`                     |
| `darling`           | 0.23.0  | 0.21.3  | pulled by `serde_with`                     |
| `darling_core`      | 0.23.0  | 0.21.3  | pulled by `darling`                        |
| `darling_macro`     | 0.23.0  | 0.21.3  | pulled by `darling`                        |
| `clap`              | 4.6.0   | 4.5.61  | edition 2024                               |
| `clap_builder`      | 4.6.0   | 4.5.61  | pulled by `clap`                           |
| `clap_derive`       | 4.6.0   | 4.5.61  | pulled by `clap`                           |
| `clap_lex`          | 1.1.0   | 1.0.1   | edition 2024                               |
| `globset`           | 0.4.18  | 0.4.15  | edition 2024                               |
| `ignore`            | 0.4.25  | 0.4.22  | required `globset ^0.4.18`                 |
| `home`              | 0.5.12  | 0.5.11  | edition 2024                               |
| `image`             | 0.25.10 | 0.25.6  | removed `moxcms`/`pxfm` (edition 2024)     |
| `png`               | 0.18.1  | 0.17.16 | pulled by `image`                          |
| `jsonwebtoken`      | 10.3.0  | 9.3.1   | direct change (see above)                  |
| `quickcheck`        | 1.1.0   | 1.0.3   | required `rand ^0.10`                      |
| `env_logger`        | 0.11.9  | 0.8.4   | pulled by `quickcheck`                     |
| `uuid`              | 1.22.0  | 1.20.0  | `getrandom 0.4` edition 2024               |
| `base64ct`          | 1.8.3   | 1.7.3   | edition 2024                               |
| `psm`               | 0.1.30  | 0.1.24  | removed `ar_archive_writer` (edition 2024) |
| `tera`              | 1.20.1  | 1.20.0  | required rustc 1.85                        |

### Removed crates

No longer in the dependency tree after the downgrades:

- `rand 0.10`, `rand_core 0.10`, `chacha20`, `cpufeatures` — pulled
  by `rand 0.10`
- `getrandom 0.4` — edition 2024
- `moxcms`, `pxfm` — pulled by `image 0.25.10`
- `ar_archive_writer` — pulled by `psm 0.1.30`

### Source changes

`rand 0.9` renames `rand::RngExt` back to `rand::Rng`. All call
sites were updated accordingly (no behavioural change).

## Consequences

- The workspace version is bumped to **3.1.0-develop**.
- The workspace MSRV is now **1.83**.
- All dependency versions are frozen in `Cargo.lock` to editions
  <= 2021.
- A future ADR will cover the actual migration to `edition = "2024"`,
  at which point these pins can be lifted and crates updated to their
  latest versions.
- A new **MSRV** CI job reads `rust-version` from `Cargo.toml` and
  verifies the workspace builds and tests pass on that exact
  toolchain.
