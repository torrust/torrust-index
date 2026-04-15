# Agent Instructions

## Cargo Commands

Prefer using `--workspace --all-targets --all-features` when running
`cargo check`, `cargo clippy`, or `cargo test`.

Be mindful to at least do a quick spot-test with `--no-default-features`,
and to try building the docs and running the doc-tests.

When testing, also run the tests in `--release` mode to check that this
works. The dev profile already sets `opt-level = 3` in `Cargo.toml`,
so debug builds are fast by default. If you need full backtraces for
debugging, temporarily override with `CARGO_PROFILE_DEV_OPT_LEVEL=0`.

When working inside a package, prefer running only the `--package` tests,
as the whole-project tests are slow to run. (Occasionally run the whole
suite, for example when finishing up.)

## Running Tests

When running tests, tee to a temp file (`/tmp/...`) and then grep that
file after the tests have completed.

## Counts and Numbers

Do not use use exact numbers, i.e. not: "100 tests passed".
Instead: put fuzzy numbers i.e. "A hundred tests passed", or "~320 functions in total".
Precise numbers may be used for qualified historical records. e.g. "on 21.12.1992 phase 3 was marked as accepted with 465 passing tests and 3 failures."

## Test Locations

We use three levels of tests based upon viability. In general, crate tests
are preferred over unit tests. Integration tests should test the public
API, perhaps using `#[doc(hidden)]` helpers when appropriate.

| Level       | Visibility   | Location      |
|-------------|--------------|---------------|
| Unit        | `private`    | inline        |
| Crate       | `pub(crate)` | `/src/tests/` |
| Integration | `pub`        | `/tests/`     |


## Test Doc-Headers

Every test file (module) should maintain an index of the tests contained in the module-doc. The primary purpose is to make it easy to scan the test files to detect duplicates or overlapping coverage. Please opportunistically create if missing.

## Cross-Reference Conventions

Eagerly corrected when spotted in **any** file!

Cross-references use the `§` (section sign) prefix. Every reference
carries a **package qualifier** so the target document is never ambiguous.
ADR references (`ADR-T-001`, `ADR-R-001`, …) are an exception — they
use their own `ADR-<PREFIX>-<NNN>` form without the `§` prefix.

| Prefix | Package              | Example document                     |
|--------|----------------------|--------------------------------------|
| `T-`   | Torrust (root crate) | *(none yet)*                         |
| `R-`   | render-text-as-image | `packages/render-text-as-image/`     |

### General Rules

- Use `§§` for ranges: e.g. `§§IDEA M-12.2–12.5`.
- Bare `§N` (no label) is acceptable **within** a document that already
  establishes context (e.g. inside `idea.md` itself), but in source
  code and cross-package references always use the fully qualified
  `§BOOK PACKAGE_PREFIX-N` form.
- Example: `§SPEC R-1.1` refers to §1.1 of `packages/render-text-as-image/docs/specification.md`.
- ADRs live in an `adr/` directory per package (`/adr/` for the
  root crate) and use sequential numbering: `NNN-slug.md`. Referenced
  as `ADR-T-001`, `ADR-R-001`, `ADR-R-002`.

## Replacing a File

To avoid partial or corrupted writes, always replace files atomically:

1. Read the file.
2. Using the CLI, `rm` the file.
3. Recreate the file.
