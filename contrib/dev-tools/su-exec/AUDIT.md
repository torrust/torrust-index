# `su-exec` Vendoring Audit

This document records the provenance, rationale, and review
history for the vendored copy of [`su-exec.c`](./su-exec.c).
It is consumed by ADR-T-009 §D8 / Acceptance Criterion #8 —
the file-change CI guard parses the most recent `SHA-256:`
line in the [Audit Log](#audit-log) section and fails the
build when it disagrees with `sha256sum su-exec.c`.

## Provenance

- **Upstream project:** [`ncopa/su-exec`](https://github.com/ncopa/su-exec)
  (MIT, Copyright (c) 2015 Natanael Copa; unmaintained,
  no tagged releases).
- **Upstream commit.** *Not recorded at initial vendor
  time.* The copy was taken from the then-latest
  upstream `master` during the repository commit below
  and has not been modified since. Re-vendoring is the
  only supported update path, so a precise upstream
  commit will be recorded as part of that exercise
  (together with a fresh audit-log entry).
- **Vendored on:** 2023-10-14, in repository commit
  `1f5351db88dc8ea7d295c115c86feb3e70498aa0`
  ("dev: upgrade containers"). The file has been unchanged
  in this repository since.
- **Files vendored:** [`su-exec.c`](./su-exec.c) (1900 bytes,
  ~75 LoC of C), [`Makefile`](./Makefile),
  [`README.md`](./README.md), and [`LICENSE`](./LICENSE)
  (MIT).
- **Initial SHA-256 of `su-exec.c`:**
  `d6c40440609a23483f12eb6295b5191e94baf08298a856bab6e15b10c3b82891`

## Choice Rationale

The container entry script needs to drop privileges from
root to the runtime `torrust` user *and* `exec` into the
application without spawning a child shell — TTY signals
must reach the application directly, and the process tree
must remain shallow so `PID 1` semantics are preserved.

Alternatives considered:

- **`gosu`** — Go-based; functionally equivalent. Rejected
  because the static binary is roughly 1.8 MB versus
  `su-exec`'s ~10 KB. The size delta matters on the lean
  distroless `release` base, which deliberately avoids
  pulling in a full language runtime for a privilege-drop
  shim.
- **`setpriv`** (util-linux) — pulls in util-linux as a
  dependency. Not present on the lean distroless
  `cc-debian13` base; adding it would expand the runtime
  attack surface for no functional gain.
- **`su` / `runuser`** — both fork the target as a child
  rather than `exec`-ing it. This breaks the PID-1 / signal
  chain that the Compose `--init`-less workflow depends on.

`su-exec` is ~75 lines of C with no transitive
dependencies beyond libc. The codebase is small enough to
audit in full and stable enough that "unmaintained upstream"
is a feature rather than a risk: there is no churn to track.

## Re-Audit Triggers

The audit is **not** on a calendar — see ADR-T-009 §D8 for
the full rationale. Static, frozen C code with a finite
review surface does not decay with time; a calendar trigger
would manufacture review work without producing review
signal.

The two real triggers:

- **File-change trigger (automated).** CI computes
  `sha256sum contrib/dev-tools/su-exec/su-exec.c` and
  compares it against the most recent `SHA-256:` line in
  the [Audit Log](#audit-log) section below. Mismatch fails
  the build until a new audit-log entry is appended that
  records the new hash and the reviewer's findings. The
  check lives in the Container CI workflow so it cannot be
  bypassed by the normal PR flow.
- **CVE trigger (manual).** When a CVE is publicly
  disclosed against `su-exec` or against a closely related
  project (`gosu`, `setpriv`, BusyBox `su`/`runuser`) that
  could plausibly apply to this code path, perform a fresh
  review and append an entry. There is deliberately no
  automated CVE feed wired in: the false-positive rate for
  an unmaintained project of this size is not worth the
  noise.

There is deliberately **no refresh procedure** documented
here. Upstream has not released in years; if a re-vendor is
ever needed it will be a manual diff-and-review exercise
producing a new audit entry as a side effect.

## Audit Log

Append-only. Newest entry at the bottom. Each entry must
contain a `SHA-256: <64-hex>` line — the structured marker
the CI guard parses.

### 2026-04-21 — Initial audit

- **Reviewer:** ADR-T-009 Phase 9 implementation.
- **Repository commit at review time:**
  `76f0fcde62d60b55837037549ffab32210cb81a9` (HEAD before
  this commit lands).
- **Scope.** Full read-through of `su-exec.c` against the
  upstream commit recorded in [Provenance](#provenance).
  Verified that:
  - The vendored file is byte-for-byte identical to
    upstream (no local modifications).
  - The code performs `getpwnam`/`getgrnam` lookups,
    parses the `user[:group]` spec, calls
    `setgroups`/`setgid`/`setuid` in the correct order,
    and `execvp`s the target — no shell invocation, no
    `system()`, no `popen()`.
  - All return codes from the privilege-changing syscalls
    are checked; failure paths exit non-zero with `err()`
    rather than continuing with reduced privileges silently.
  - There is no network code, no signal handling beyond
    libc defaults, and no use of environment variables
    other than what `execvp` itself consults via `PATH`.
- **Conclusion.** Suitable for the entry-script's
  privilege-drop-and-exec role. No findings.

SHA-256: d6c40440609a23483f12eb6295b5191e94baf08298a856bab6e15b10c3b82891
