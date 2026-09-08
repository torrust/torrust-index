# Torrust Index Release Process (v2.2.2)

## Version

> **The `[semantic version]` is bumped according to releases, new features, and breaking changes.**
>
> *The `develop` branch uses the (semantic version) suffix `-develop`.*

This process releases the application, and the `[semantic version]` it moves is the root `torrust-index` crate's `version` in the workspace root `Cargo.toml` — the only crate that carries the `-develop` suffix and the only crate this process publishes. Every other workspace crate carries its own version and is published on its own cadence; see [Publishing a Workspace Package](#publishing-a-workspace-package) below and ADR-T-012 (`adr/012-independent-package-versioning.md`).

## Process

**Note**: this guide assumes that the your git `torrust` remote is like this:

```sh
git remote show torrust
```

```s
* remote torrust
  Fetch URL: git@github.com:torrust/torrust-index.git
  Push  URL: git@github.com:torrust/torrust-index.git
...
```

### 1. The `develop` branch is ready for a release

The `develop` branch should have the root crate's version `[semantic version]-develop` ready to be released. Any workspace crate whose version changed during the cycle should already have been published; see [Publishing a Workspace Package](#publishing-a-workspace-package).

### 2. Stage `develop` HEAD for merging into the `main` branch

```sh
git fetch --all
git push --force torrust develop:staging/main
```

### 3. Create Release Commit

```sh
git stash
git switch staging/main
git reset --hard torrust/staging/main
# change the root crate's `version` in `Cargo.toml` from `[semantic version]-develop` to `[semantic version]`. No other crate's version is touched.
git add -A
git commit -m "release: version [semantic version]"
git push torrust
```

### 4. Create and Merge Pull Request from `staging/main` into `main` branch

Pull request title format: "Release Version `[semantic version]`".

This pull request merges the new version into the `main` branch.

### 5. Push new version from `main` HEAD to `releases/v[semantic version]` branch

```sh
git fetch --all
git push torrust main:releases/v[semantic version]
```

> **Check that the deployment is successful!**

### 6. Create Release Tag

```sh
git switch releases/v[semantic version]
git tag --sign v[semantic version]
git push --tags torrust
```

Make sure the [deployment](https://github.com/torrust/torrust-index/actions/workflows/deployment.yaml) workflow was successfully executed and the new version of the one crate it publishes is on the registry:

- [torrust-index](https://crates.io/crates/torrust-index)

The workflow triggers on `releases/v*` and publishes only the root crate. The workspace crates it depends on are expected to be on crates.io already, published independently as they evolved.

### 7. Create Release on Github from Tag

This is for those who wish to download the source code.

### 8. Stage `main` HEAD for merging into the `develop` branch

Merge release back into the develop branch.

```sh
git fetch --all
git push --force torrust main:staging/develop
```

### 9. Create Comment that bumps next development version

```sh
git stash
git switch staging/develop
git reset --hard torrust/staging/develop
# change the root crate's `version` in `Cargo.toml` from `[semantic version]` to `(next)[semantic version]-develop`. No other crate's version is touched.
git add -A
git commit -m "develop: bump to version (next)[semantic version]-develop"
git push torrust
```

### 10. Create and Merge Pull Request from `staging/develop` into `develop` branch

Pull request title format: "Version `[semantic version]` was Released".

This pull request merges the new release into the `develop` branch and bumps the version number.

## Publishing a Workspace Package

Every workspace crate carries its own `version` and is published on its own cadence (ADR-T-012, `adr/012-independent-package-versioning.md`). A crate does not wait for an application release to reach its consumers, and an application release does not publish it.

### Branch and Tag Conventions

| Concept        | Convention                            | Example                                    |
| -------------- | ------------------------------------- | ------------------------------------------ |
| Release branch | `releases/pkg/<crate-name>/v<semver>` | `releases/pkg/torrust-index-config/v0.1.0` |
| Release tag    | `pkg/<crate-name>/v<semver>` (signed) | `pkg/torrust-index-config/v0.1.0`          |

Pushing a branch under `releases/pkg/` runs the `Deployment (Packages)` workflow, which tests the named crate and publishes it. The application release branch `releases/v[semantic version]` has a different shape and runs a different workflow: a `*` glob does not cross a `/`, so no branch can match both patterns.

### When to Publish a Package

Whenever a crate's version has changed — a fix, a new API, a first release, or a release made so a downstream project can depend on the crate directly.

Publishing order follows the dependency graph. A crate cannot be published before the siblings it depends on are on crates.io, because the requirement `cargo publish` writes beside each `path` dependency has to resolve against the registry. `torrust-index-cli-common`, `torrust-index-config`, `torrust-index-entry-script`, `torrust-index-render-text-as-image` and `torrust-mudlark` depend on no sibling and can be published at any time. `torrust-index-auth-keypair` and `torrust-index-health-check` need `torrust-index-cli-common` first; `torrust-index-config-probe` needs both `torrust-index-cli-common` and `torrust-index-config`. `cargo publish --dry-run` reports an unmet order precisely, so run it when in doubt.

### Automated Path

1. Bump the crate's `version` in its own `Cargo.toml`, and update every manifest that pins it — the `version` beside `path` in each dependent, the workspace root included.

2. Check the crate on its own:

   ```sh
   cargo test -p [crate name] --all-targets --all-features
   ```

3. Push the release branch from `develop`:

   ```sh
   git fetch --all
   git push torrust develop:releases/pkg/[crate name]/v[semantic version]
   ```

4. The `Deployment (Packages)` workflow runs the crate's tests on nightly and stable, verifies the crate declares its own version rather than inheriting one, and publishes it to crates.io.

5. Once the workflow has succeeded, create the signed tag:

   ```sh
   git fetch --all
   git switch releases/pkg/[crate name]/v[semantic version]
   git tag --sign pkg/[crate name]/v[semantic version]
   git push --tags torrust
   ```

A package publish creates no GitHub release. GitHub releases carry the application binary; a workspace crate's distribution surface is crates.io, its README and its `Cargo.toml` metadata.

### Manual Fallback

When the workflow is unavailable, or a publish has to happen without creating a git reference:

1. Confirm the crate declares its own `version`.

2. Check the crate on its own:

   ```sh
   cargo test -p [crate name] --all-targets --all-features
   ```

3. Rehearse the publish. This is also what reports an unmet dependency order, because the dry run resolves the crate's sibling requirements against the registry rather than against the workspace:

   ```sh
   cargo publish -p [crate name] --dry-run
   ```

4. Publish:

   ```sh
   cargo publish -p [crate name]
   ```

### Worked Example

`torrust-index-config` and `torrust-index-config-probe` have never been published, and an operator tool wants to read the index configuration the way the container does.

The probe depends on the configuration crate, so the configuration crate goes first:

```sh
git push torrust develop:releases/pkg/torrust-index-config/v0.1.0
# the workflow publishes torrust-index-config 0.1.0
git switch releases/pkg/torrust-index-config/v0.1.0
git tag --sign pkg/torrust-index-config/v0.1.0 && git push --tags torrust
```

The probe also depends on `torrust-index-cli-common`, which is published the same way. Only then can the probe itself go out:

```sh
git push torrust develop:releases/pkg/torrust-index-config-probe/v0.1.0
```

Later, a fix lands in the configuration schema. Its version moves to `0.1.1` and it is published again, on its own; no other crate's version moves, and the application does not release. The workspace keeps building against the local `path` copy throughout — the published version is what an outside consumer resolves, not what this repository compiles.

When the application release finally happens, it bumps only the root crate's version and publishes only `torrust-index`. The configuration crate, the probe and the shared CLI crate are already on crates.io from their own releases.
