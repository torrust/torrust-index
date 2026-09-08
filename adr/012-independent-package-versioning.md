# ADR-T-012: Independent Package Versioning

**Status:** Decided and implemented
**Date decided:** 2026-09-08
**Date implemented:** 2026-09-08
**Relates to:** [ADR-T-005](./005-edition-2024.md) (set the shared workspace version to `4.0.0-develop`), [ADR-T-011](./011-msrv-policy.md) (the other field every crate inherits from `[workspace.package]`)

---

## Context

The workspace holds one binary crate, six helper crates, and two generic libraries. `torrust-index` is the application. `torrust-index-cli-common`, `torrust-index-config`, `torrust-index-config-probe`, `torrust-index-auth-keypair`, `torrust-index-health-check` and `torrust-index-entry-script` exist to support it and ship alongside it in the container image. `torrust-index-render-text-as-image` and `torrust-mudlark` are general-purpose libraries that do not depend on the index at all and are useful to consumers who will never run it.

The root crate and six members took their version from `[workspace.package]`, so seven crates advertised the single number `4.0.0-develop`. Two crates already stood outside it: `torrust-mudlark` declared `1.0.0` and `torrust-index-render-text-as-image` declared `0.1.0`, each a number chosen for that crate rather than inherited from the workspace.

Only `torrust-index` has ever been published. It is on crates.io at `3.0.0`, developing `4.0.0`. Every other crate in the workspace has never been released, so the versions they carried were never resolved by any consumer and never appeared in anyone's lock file.

A single shared version has four costs in this workspace.

It carries no information. A version is a claim about one crate's history, and a number that moves because an unrelated crate changed cannot support that claim. A consumer looking at `torrust-index-config` could not tell from its version whether anything in the configuration schema had changed, only that the workspace had cut a release.

It couples release cadence to the application. A fix in a generic library cannot reach crates.io without an application release, even though the library's consumers have no interest in the index. That is a bottleneck imposed by the manifest layout rather than by anything about the code.

It overstates maturity. `4.0.0-develop` on a crate with no released API claims three majors of compatibility history that never happened, which is a promise the crate is not in a position to keep.

It cannot be published. `cargo publish` reads the `version` requirement that sits beside `path` in a sibling dependency and writes it into the published manifest, because a consumer downloading the crate from the registry has no path to resolve. Publishing `torrust-index-config-probe` today would emit a requirement on `torrust-index-cli-common 4.0.0-develop`, a version that does not exist on the registry and never will. The shared number is therefore not merely uninformative — it makes every member crate but the root unpublishable as written.

## Decision

**Every crate in the workspace declares its own `version`. `[workspace.package]` carries none.**

### Version assignment

| Crate                                | Version         | Basis                                                  |
| ------------------------------------ | --------------- | ------------------------------------------------------ |
| `torrust-index`                      | `4.0.0-develop` | published at `3.0.0`, developing `4.0.0`               |
| `torrust-index-auth-keypair`         | `0.1.0`         | never published; inherited the shared version          |
| `torrust-index-cli-common`           | `0.1.0`         | never published; inherited the shared version          |
| `torrust-index-config`               | `0.1.0`         | never published; inherited the shared version          |
| `torrust-index-config-probe`         | `0.1.0`         | never published; inherited the shared version          |
| `torrust-index-entry-script`         | `0.1.0`         | never published; inherited the shared version          |
| `torrust-index-health-check`         | `0.1.0`         | never published; inherited the shared version          |
| `torrust-index-render-text-as-image` | `0.1.0`         | never published; keeps the version it already declared |
| `torrust-mudlark`                    | `1.0.0`         | never published; keeps the version it already declared |

The assignment follows three rules.

**A crate that is on crates.io carries forward the release line it is developing.** `torrust-index` is published at `3.0.0` and is working towards `4.0.0`, so it keeps `4.0.0-develop` and is released as `4.0.0`. It is the only crate in the workspace that carries the `-develop` pre-release suffix, because it is the only one whose version tracks an application release cycle.

**A crate that inherited the shared version and has never been published starts at `0.1.0`.** For the six helper crates this lowers the declared number. That regression is permitted and expected. A version no consumer ever resolved signalled nothing, so restating it breaks no dependency, invalidates no lock file, and contradicts no published artefact; there is nothing on the registry for the new number to be compared against. And `0.x` is the honest statement of an API that has not been released: under SemVer the `0.x` line is exactly the space reserved for an interface that is still free to change, which is the position every one of these crates is actually in.

**A crate that already declared its own version keeps it.** `torrust-mudlark` stays at `1.0.0` and `torrust-index-render-text-as-image` stays at `0.1.0`. Neither number was inherited, so neither is the shared number's problem: each is the crate's own statement about its own API, made deliberately by the crate that has to keep it. What this policy removes is a version that says nothing about the crate carrying it, and a crate that already declared its own has done what the policy asks. Lowering such a number would apply a reason that holds only for the shared one — `torrust-mudlark` documents the surface its guarantee covers and records the release at which the guarantee began, and being absent from the registry does not retract that. Publication is distribution; a compatibility promise is made by the crate, not by the registry.

### Path dependencies carry the sibling's version

A dependency on a workspace sibling pins that sibling's explicit version beside its path:

```toml
torrust-index-config = { version = "0.1.0", path = "packages/index-config" }
```

The `path` is what the workspace build resolves, always and regardless of the number. The `version` is what `cargo publish` writes into the published manifest, and it is the only thing a consumer downloading the crate from crates.io has to go on. The two are updated together at every bump, and a pin that names a version the sibling does not declare is a publishing defect even though the workspace build never notices it.

### When versions move

A crate's version moves when that crate changes, under SemVer, and for no other reason. A change in one crate does not move any other crate's number.

The root crate's version additionally moves with the application release cycle, as it always has: `-develop` on the development branch, the bare version at release.

### How a crate is published

Publication is per crate.

| Concept             | What it publishes                         | Branch                                | Tag                                   | Workflow                    |
| ------------------- | ----------------------------------------- | ------------------------------------- | ------------------------------------- | --------------------------- |
| Application release | Only `torrust-index`                      | `releases/v<semver>`                  | `v<semver>` (signed)                  | `deployment.yaml`           |
| Package publish     | Exactly one workspace crate               | `releases/pkg/<crate-name>/v<semver>` | `pkg/<crate-name>/v<semver>` (signed) | `deployment-packages.yaml`  |

The two branch patterns are mutually exclusive without negative matching, because a glob `*` does not cross a `/` boundary: `releases/v*` cannot match `releases/pkg/...`.

The application release publishes only the root crate. Every crate it depends on is expected to be on crates.io already, published independently as it evolved; a release is not the moment to discover that a helper crate has never been uploaded.

Publishing order still binds: a crate cannot be published before the siblings it depends on are on the registry, because the registry has to be able to resolve the requirement the published manifest carries. That order is a property of the dependency graph, not of this policy, and `cargo publish --dry-run` reports it precisely.

### What the policy does not do

It does not decouple the build. Sibling dependencies keep their `path`, so a workspace build, test or lint always compiles the local copy and never the registry one. Independent version numbers change what is published, not what is compiled.

It does not make every crate immediately publishable in any order. The dependency graph still has to be walked from the leaves.

It does not withhold any crate from the registry. Setting `publish = false` is the right answer for a crate that exists only to serve the repository's own tooling, and this workspace has none in that class: the helper crates ship in the container image and are usable on their own terms, and the two generic libraries are useful to consumers with no interest in the index at all.

It does not turn a crate's version into a statement about a contract shared with another crate. The public contracts of this workspace are already versioned in the sources, in namespace modules a consumer can read: the REST API under `src/web/api/server/v1/` and the configuration schema under `packages/index-config/src/v2/`. The manifest version tracks the crate's own release history, which is a distribution concern and a different question.

## Alternatives considered

| Alternative                                                | How it assigns versions                                                     | Why not                                                                                                                                                                                                                                                             |
| ---------------------------------------------------------- | ---------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| A. One shared workspace version (the status quo)           | Every crate inherits one number from `[workspace.package]`                   | Carries no per-crate information, couples every library's cadence to the application, overstates maturity, and — decisively — cannot be published at all, because the sibling requirements it writes into published manifests name a version no registry copy has |
| B. Two tiers: the application and its helpers linked, the generic libraries independent | One number for the index and its six helpers, own numbers for `torrust-mudlark` and `torrust-index-render-text-as-image` | The link buys nothing the `path` dependency does not already guarantee, and it freezes a guess about coupling: a helper crate that is later extracted or gains an outside consumer has to leave the tier, which is a migration this policy would have avoided |
| C. Link the versions of crates that share a contract       | `torrust-index-config` and `torrust-index-config-probe` move together; the crates carrying the command-line output contract move together | The shared thing is a schema, and a schema is versioned in the sources where consumers can read it, not in a manifest field they see only when resolving a dependency. Linking would move the configuration crate's number for a fix in the probe's argument handling, which is precisely the noise this decision removes |
| D. A continuous-integration gate that rejects `version.workspace = true` | Nothing — it forbids a shape without saying what number a crate should carry | Not an alternative to the policy but a consequence of it: the check is worth having, and it now runs immediately before the publish step of the package publishing workflow, where a failure blocks the only action it could corrupt |
| E. Every crate versions independently (**chosen**)         | Each crate's own release history                                             | The version means what SemVer says it means, each crate releases when it has something to release, and every published manifest carries a requirement the registry can resolve                                                                                       |

## Consequences

- `[workspace.package]` no longer carries `version`, and every crate — the root included — declares its own. A member manifest that reintroduces `version.workspace = true` is rejected before it can be published.
- Sibling pins beside `path` now name versions that will exist on crates.io once the crates are published, so `cargo publish` emits resolvable requirements.
- The two crates that had already declared their own versions are untouched: `torrust-mudlark` stays at `1.0.0` and `torrust-index-render-text-as-image` at `0.1.0`. Only the crates that inherited the shared number move, so no manifest outside that set changed and no crate's documentation needed correcting.
- Versions a crate declared before this policy remain in the changelogs that recorded them. Those entries are records of what the sources said at the time, not claims about the present, in the same way historical ADRs keep the numbers they recorded.
- The deployment workflow narrows to `releases/v*` and continues to publish only `torrust-index`; a new packages workflow publishes exactly one crate from a `releases/pkg/**` branch. The container workflow narrows to `releases/v*` as well, so a package release does not start an image build that can produce nothing.
- The release process gains a package publishing path, and its application release steps say explicitly that only the root crate's version is bumped.
- A first publish of any helper or library crate has to follow the dependency order. `cargo publish --dry-run` on a crate whose siblings are not yet on the registry fails, and that failure is the policy reporting the publication order correctly rather than a defect to work around.

## References

- The sibling project reached the same decision for the same reasons: torrust/torrust-tracker [#1926](https://github.com/torrust/torrust-tracker/issues/1926) defined the strategy, its ADR *Adopt Independent Package Versioning* recorded it, and [#1961](https://github.com/torrust/torrust-tracker/pull/1961) migrated its workspace off the shared `3.0.0-develop`. That workspace is larger and sorts its crates into versioning-semantics tiers; this one has a single binary, a handful of helper crates and two generic libraries, so it needs the rule without the taxonomy.
