# ADR-S-004: Config Validation Over Panic · `rec:sentinel:structured-prevalidation-before-mudlark-construction`

**Status:** Decided **Date:** 2026-03-09 **Spec:** §ALGO S-13.3 (config constraints) **Relates to:** [ADR-S-003](003-mudlark-integration.md) (mudlark integration)

## Context · `sec:sentinel:configguard-context`

Mudlark's `GvGraph::new(config)` internally calls `config.validate()`, which uses `assert!` to enforce constraints such as:

- `split_threshold > 0`
- `depth_evict > depth_create`
- `budget > max(3^(buffer+1), 2*(depth_create − 1))` where `buffer = depth_evict − depth_create`

If any constraint is violated, the process panics. This is acceptable for mudlark as a data-structure library — callers are expected to pass valid configs, and panicking on programmer error is idiomatic Rust.

The sentinel, however, is a library consumed by the Torrust Index application. Panicking on bad user configuration is unacceptable:

1. The Torrust process would abort if a TOML config file contained an invalid `d_create` / `d_evict` combination.
2. The caller has no opportunity to report the error, retry, or fall back to defaults.
3. Multiple config errors cannot be collected — `assert!` fires on the first violation and halts.

## Decision · `sec:sentinel:configguard-decision`

**The sentinel re-derives mudlark's config constraints in its own `SentinelConfig::validate()` method and returns `Result<(), ConfigErrors>` where `ConfigErrors` is a list of `ConfigError` variants.**

The sentinel's validation runs *before* `GvGraph::new()` is ever called, so mudlark's `assert!` paths are unreachable under normal operation. The redundancy is intentional.

### Headroom formula · `sec:sentinel:configguard-headroom-formula`

The most complex constraint is the budget headroom check:

```rust
let buffer = self.d_evict - self.d_create;
let headroom = 3usize.pow(buffer + 1);
let convergence = 2 * (self.d_create as usize).saturating_sub(1);
let required = headroom.max(convergence);
if self.budget <= required {
    errors.push(ConfigError::BudgetTooSmall {
        budget: self.budget,
        required_minimum: required,
    });
}
```

This mirrors mudlark's internal calculation. A comment in the sentinel code cross-references the mudlark source so that future changes to the formula can be synchronised.

## Alternatives Considered · `sec:sentinel:configguard-alternatives`

- **Catch the panic.** `std::panic::catch_unwind()` around `GvGraph::new()`. Rejected — fragile, opaque (string error), cannot collect multiple violations, and `catch_unwind` is a last resort in idiomatic Rust.

- **Ask mudlark to return `Result`.** A longer-term option. Even then, sentinel's pre-validation remains useful — it collects *all* errors at once and produces sentinel-specific error types for the UI.

- **Trust the caller.** Document constraints and `debug_assert!` only. Rejected — config comes from user-edited TOML; any typo could crash the server.

## Consequences · `sec:sentinel:configguard-consequences`

- Callers receive structured, actionable `ConfigError` variants listing every constraint violation, not a single panic message.
- If mudlark changes its headroom formula, the sentinel's re-derivation must be updated. The comment `// mirrors mudlark Config::validate()` marks this coupling.
- Mudlark's own `assert!` paths become dead code in production — they serve only as a defence-in-depth safety net.

### Panic-vs-Result boundary · `sec:sentinel:configguard-panic-result-boundary`

The ADR's `Result` policy applies to `SentinelConfig` parameters that originate from user-edited TOML. Decay parameters (`att`, `q`) passed to `decay()` and `decay_subtree()` are programmer-controlled call-site values. Invalid decay parameters (negative `att`, `q` outside $[0, 1]$) trigger panics, not `Result` — these are programming errors, not user-input errors. The boundary is: **config = `Result`, call-site = panic**.

### Depth 128 rejection · `sec:sentinel:configguard-depth-128-rejection`

At G-tree depth $N$, suffix width $w = N - N = 0$, making the tracker dimensionless — no suffix bits remain for statistical analysis. The config validator rejects depth configurations that would allow depth-$N$ cells to enter the analysis set, constraining the effective analysis depth range to $[1, N-1]$.
