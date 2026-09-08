# ADR-T-011: Minimum Supported Rust Version Policy

**Status:** Decided and implemented
**Date decided:** 2026-09-05
**Date implemented:** 2026-09-05
**Relates to:** [ADR-T-003](./003-edition-2024-preparation.md) (MSRV 1.80 → 1.83), [ADR-T-005](./005-edition-2024.md) (MSRV 1.83 → 1.88)

---

## Context

The workspace has raised its minimum supported Rust version three times, and every raise was reactive. ADR-T-003 moved the floor from 1.80 to 1.83 because the transitive dependency tree would not build below it. ADR-T-005 moved it from 1.85 to 1.88 because `time 0.3.47` demanded 1.88, even though edition 2024 itself needs only 1.85. In each case the number was discovered rather than chosen: the workspace asked its dependency tree what the floor had to be and wrote down the answer.

That has three costs.

The first is unpredictability for downstream builders. Distributions ship a frozen toolchain, and operators who build from source often pin one. A floor that can jump whenever an unrelated crate publishes a release gives them nothing to plan against — neither the timing of the next raise nor its size.

The second is that the schedule belongs to third parties. A single dependency deciding to use a newly stabilised feature moves the floor of this workspace, for no benefit to this workspace. The floor then also tends to sit higher than anything in the first-party sources actually requires.

The third is that every bump is argued from scratch. Because there is no rule, each raise needs its own justification, its own review, and its own record — a recurring negotiation about a number that carries no design content.

The dependency-driven floor also conflates two separate questions that have separate answers. "Which toolchains do we promise to support?" is a compatibility commitment to downstream consumers. "Which dependency versions can we resolve?" is a build-time selection problem, and since the migration to edition 2024 the MSRV-aware resolver already solves it: given a declared `rust-version`, Cargo selects the newest dependency versions compatible with that floor. A dependency that wants a newer toolchain is a reason to select an older release of that dependency, not a reason to move the promise.

## Decision

**The workspace `rust-version` is the newest stable Rust release that was published at least one year before the day the pin is computed.**

The floor is computed, not negotiated. It is a function of the calendar and the Rust release history alone: no dependency requirement, no feature wish, and no reviewer preference enters it. The floor only ever rises.

### Computation on 2026-09-05

| Release | Published  | At least one year old on 2026-09-05? |
| ------- | ---------- | ------------------------------------ |
| 1.89.0  | 2025-08-07 | Yes — newest such release            |
| 1.90.0  | 2025-09-18 | No                                   |

The pin is therefore **1.89**, set once in `[workspace.package]` and inherited by every member crate.

### When the pin is recomputed

- At every release of the workspace.
- In any maintenance pass that touches dependencies or the toolchain.

A recomputation that yields the current pin is a no-op and needs no record. A recomputation that raises the pin is recorded in the changelog as a breaking change, naming this policy as the reason. It does not need an ADR of its own: the rule was decided here, and applying a rule is not a decision.

### What the policy does not do

It does not require the workspace to build on the floor toolchain by accident — the MSRV job in continuous integration reads `rust-version` from `Cargo.toml` and runs the build checks on exactly that toolchain, so the promise is enforced rather than asserted.

It does not constrain development. Contributors build, test and lint on current stable or nightly; the MSRV job is the guard that catches a construct the floor cannot compile.

It does not make a dependency's requirement into a reason to raise the floor. When a dependency requires more than the computed floor, the response is to hold that dependency at its newest floor-compatible release — which the resolver does on its own — or, where that is untenable, to replace the dependency.

## Alternatives considered

| Alternative                                            | How it sets the floor                                                | Why not                                                                                                                                                                                       |
| ------------------------------------------------------ | -------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| A. Dependency-driven floor (the status quo)            | Whatever the transitive tree demands                                 | Unpredictable in timing and size; hands the schedule to third parties; conflates a compatibility promise with a resolution problem the resolver already solves                                 |
| B. Track latest stable                                 | The newest release                                                   | Excludes every distribution and every pinned build; makes the workspace unbuildable for consumers who are doing nothing wrong                                                                  |
| C. A fixed number of releases behind latest            | Latest minus N                                                       | Expressed in a unit downstream consumers do not plan in; with a six-week cadence it approximates the calendar rule anyway, so it is the same policy stated less legibly                        |
| D. A pinned floor changed only by explicit decision    | Whatever was last agreed                                             | The status quo under a different name: it reintroduces a per-bump negotiation and leaves downstream with no way to anticipate the next move                                                    |
| E. Newest stable at least one year old (**chosen**)    | The calendar                                                         | Predictable a year ahead, computable by anyone without consulting the maintainers, independent of dependency churn, and stated in the unit distributions actually use                          |

## Consequences

- `workspace.package.rust-version` is raised from `1.88` to `1.89`. Every member crate inherits it through `rust-version.workspace = true`.
- This is a breaking change for consumers on a toolchain older than 1.89, recorded as such in the changelog.
- The MSRV job in `.github/workflows/testing.yaml` needs no change: it already extracts `rust-version` from `Cargo.toml` and installs that toolchain, so the raise propagates from the manifest alone.
- Any consumer whose toolchain is at most one year old can build the workspace. That window is guaranteed forward: a toolchain that works today keeps working for a year.
- Raising the floor is no longer an argument. A maintenance pass recomputes it, and the only judgement left is whether the sources compile — which the MSRV job answers.
- Stale MSRV references in prose (the README, the crate-level documentation, the `mudlark` package README) are corrected as part of the raise, so the manifest and the documentation cannot drift apart.
- Historical ADRs keep the numbers they recorded. ADR-T-003 and ADR-T-005 document what the floor was when those decisions were made; they are records, not statements about the present.
