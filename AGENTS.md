# Agent Instructions

## Cargo Commands

Prefer using `--workspace --all-targets --all-features` when running
`cargo check`, `cargo clippy`, or `cargo test`.

Be mindful to at least do a quick spot-test with `--no-default-features`,
and to try building the docs and running the doc-tests.

When testing, also run the tests in `--release` mode to check that this
works. Run the tests in debug mode with `CARGO_PROFILE_DEV_OPT_LEVEL=3`;
otherwise it is just too slow. You can turn off the optimisation if a bug
is found and you need backtracing.

When working inside a package, prefer running only the `--package` tests,
as the whole-project tests are slow to run. (Occasionally run the whole
suite, for example when finishing up.)

## Running Tests

When running tests, tee to a temp file (`/tmp/...`) and then grep that
file after the tests have completed.

## Test Locations

We use three levels of tests based upon viability. In general, crate tests
are preferred over unit tests. Integration tests should test the public
API, perhaps using `#[doc(hidden)]` helpers when appropriate.

| Level       | Visibility   | Location      |
|-------------|--------------|---------------|
| Unit        | `private`    | inline        |
| Crate       | `pub(crate)` | `/src/tests/` |
| Integration | `pub`        | `/tests/`     |

## Cross-Reference Conventions

Eagerly corrected when spotted in **any** file!

Cross-references use the `§` (section sign) prefix. Every reference
carries a **package qualifier** — `T-` for Torrust —
so the target document is never ambiguous.

### General Rules

- Use `§§` for ranges: e.g. `§§ALGO M-12.2–12.5`.
- Bare `§N` (no label) is acceptable **within** a document that already
  establishes context (e.g. inside `algorithm.md` itself), but in source
  code and cross-package references always use the fully qualified
  `§BOOK PACKAGE_PREFIX-N` form.

## Replacing a File

1. Read the file.
2. Using the CLI, `rm` the file.
3. Recreate the file.
