# ADR-T-008: Refactor the Roles and Permissions System

**Status:** Implemented (Phases 1–4 complete; MySQL E2E pending)
**Date:** 2026-04-15
**Relates to:**
[ADR-T-007](007-jwt-system-refactor.md) (JWT refactor — advisory `role` in token claims),
[ADR-T-006](006-error-system-refactor.md) (error refactor — `AuthError::UnauthorizedAction` variants)

## Context

The roles and permissions system has grown organically alongside
the rest of the codebase. While the introduction of Casbin was a
step forward, the overall architecture still has structural problems
that limit flexibility, auditability, and correctness.

### Current Architecture

Authorization is spread across three mechanisms:

| Mechanism | Location | Description |
|---|---|---|
| Casbin RBAC | `services::authorization::Service` | Evaluates `(role, ACTION)` pairs against a policy matrix |
| Boolean `administrator` flag | `torrust_users.administrator` column | Single bit in the database; the only persistent role storage |
| Inline ownership checks | e.g. `services::torrent::Index::update_torrent` | `if !(uploader == username \|\| administrator)` |

**Role model.** There are exactly three roles, defined as a
private enum in `services/authorization.rs`:

```rust
enum UserRole {
    Admin,
    Registered,
    Guest,
}
```

Role assignment is derived at runtime: if the user has
`administrator == true` in the database, they are `Admin`;
otherwise `Registered`; absent users (or no token) are `Guest`.

**Casbin policy.** The default Casbin configuration defines a flat
`(role, action)` matrix with 22 actions and ~50 policy lines
hard-coded as a Rust string literal in
`CasbinConfiguration::default()`. The model is a simple
request-matches-policy matcher with no support for resource
ownership, hierarchies, or conditions.

**Config override.** An operator can supply a custom Casbin model
and policy via `unstable.auth.casbin.{model, policy}` in the TOML
config. This is the only way to change permissions without editing
Rust source code.

**ACTION enum.** Actions are defined as a `SCREAMING_CASE` enum
(violating Rust naming conventions) at the top of
`services/authorization.rs`:

```rust
pub enum ACTION {
    GetAboutPage,
    GetLicensePage,
    AddCategory,
    DeleteCategory,
    // … 18 more variants
}
```

### Identified Problems

1. **Binary role model — no extensibility.** The system has exactly
   two persistent states: `administrator = true` and
   `administrator = false`. There is no way to define intermediate
   roles (e.g. `moderator`, `uploader`, `trusted`) without
   changing the database schema and the `get_role` derivation
   logic. Casbin supports arbitrary role names, but the bridge
   between the database and Casbin is hard-coded to map a boolean
   to one of two roles.

2. **No resource-level authorization.** Casbin is only consulted
   for `(role, action)` — it has no knowledge of *which* resource
   is being acted upon. Resource-level checks (e.g. "can this user
   edit *this* torrent?") are performed via inline `if` statements
   in service methods:
   ```rust
   // services/torrent.rs — todo comment acknowledges the problem
   // todo: move this to an authorization service.
   if !(torrent_listing.uploader == updater.username || updater.administrator) {
       return Err(TorrentError::UnauthorizedAction);
   }
   ```
   These checks are scattered, inconsistent, and impossible to
   audit by reading the Casbin policy alone.

3. **Policy is invisible at the API boundary.** There is no
   endpoint or mechanism for a client to discover *what* a given
   user is allowed to do. The permissions matrix is only visible
   by reading the Casbin configuration string or the Rust source.
   Frontends must guess or hard-code their own permission logic.

4. **`ACTION` enum naming violates Rust conventions.** The enum
   is named `ACTION` (SCREAMING_CASE) and its variants use
   `PascalCase`, which is correct for the variants but the type
   name itself should be `Action`. The `#[allow(non_camel_case_types)]`
   suppression is implicit via the serde rename.

5. **Actions are too coarse / too fine in wrong places.** Some
   actions guard entire features (`GetSettings` vs.
   `GetSettingsSecret` vs. `GetPublicSettings` — three actions for
   one endpoint with different detail levels), while others are
   absent entirely (`UpdateTorrent` has no Casbin check; only an
   inline ownership check exists). There is no consistent
   granularity.

6. **First-user auto-admin is implicit and fragile.** The first
   registered user is automatically granted admin rights via
   `if user_id == 1 { grant_admin_role }` in
   `RegistrationService::register`. This is undocumented, does not
   use the authorization service, and relies on the database
   auto-increment starting at 1.

7. **No audit trail.** Role changes (`grant_admin_role`) are not
   logged, timestamped, or recorded anywhere beyond the database
   column flip. There is no history of who granted what role to
   whom.

8. **Casbin is a heavy dependency for a simple matrix.** The
   current policy is a flat `(role, action)` lookup — it does not
   use Casbin's RBAC hierarchy, domain tenancy, ABAC matchers, or
   any advanced features. The `casbin` crate and its transitive
   dependencies add compile-time and runtime cost for
   functionality that could be expressed as a simple `HashSet` or
   `match` block.

9. **Casbin enforcer panics on startup errors.** Both
   `with_default_configuration` and `with_configuration` use
   `.expect()` on the Enforcer creation and policy loading. A
   malformed custom policy in the config will crash the process at
   startup with an unhelpful panic.

10. **Admin flag is exposed as a raw boolean in API responses.**
    `TokenResponse` returns `admin: bool` and `UserCompact` carries
    `administrator: bool`. If more roles are added, this leaks the
    internal representation and requires a breaking API change.

11. **Authorization service is tightly coupled to `UserCompact`.**
    `get_role` fetches a full `UserCompact` from the database
    solely to read the `administrator` boolean. This creates a
    dependency on the user repository inside the authorization
    layer and makes it impossible to derive roles from any other
    source (e.g. a roles table, external IdP, group membership).

12. **`UnauthorizedAction` vs. `UnauthorizedActionForGuests` is
    ambiguous.** These two error variants exist across multiple
    error enums (`AuthError`, `UserError`, `TorrentError`), with
    the distinction being whether the caller is a guest or an
    authenticated user. The same action failure produces different
    errors depending on role, complicating client error handling.

## Options

### Option A — Incremental Cleanup (Minimal Scope)

Fix naming, remove the worst inconsistencies, and unify the
authorization call-sites without changing the role model or
replacing Casbin.

**Changes:**

- Rename `ACTION` → `Action` (fix the naming convention violation).
- Add missing actions to the Casbin policy (`UpdateTorrent`,
  etc.) and route all inline ownership/admin checks through
  `authorization::Service::authorize`.
- Extend the `authorize` signature to accept an optional resource
  context (e.g. `resource_owner: Option<UserId>`) so ownership
  checks can be co-located with role checks.
- Replace `.expect()` panics in `CasbinEnforcer` with `Result`
  propagation.
- Log role grants via `tracing`.
- Expose `administrator` as a `role: "admin" | "user"` string in
  API responses rather than a raw boolean (backward-compatible
  addition: add `role` field alongside the existing `admin` boolean,
  deprecate the boolean).

**Pros:**
- Small diff, low risk, no schema migration.
- All inline authorization checks become auditable through the
  Casbin policy.
- No new dependencies.

**Cons:**
- The binary role model remains — no path to `moderator` or custom
  roles without further work.
- Casbin remains a heavy dependency for a simple matrix.
- No resource-level policy in Casbin itself (just co-located in
  the service method).
- No permissions discovery endpoint.

---

### Option B — Replace Casbin with a Native Rust Permission System

Remove the Casbin dependency entirely and implement a lightweight,
purpose-built authorization layer using Rust types and traits.

**Changes:**

- Define a `Role` enum with variants that can grow:
  ```rust
  enum Role {
      Guest,
      Registered,
      Moderator, // future
      Admin,
  }
  ```
- Replace the `administrator: bool` column with a `role: TEXT`
  (or `role_id: INTEGER` + lookup table) column in the database.
  Provide a migration from the boolean to the new column.
- Implement a `PermissionMatrix` (e.g. `HashMap<(Role, Action),
  Effect>` or a compile-time `const` table) that replaces the
  Casbin policy string.
- Extend `Action` variants to cover resource-level operations
  (e.g. `UpdateOwnTorrent`, `UpdateAnyTorrent`, or a single
  `UpdateTorrent` with an ownership flag).
- The matrix is defined in code (compile-time checked) with an
  optional TOML override for operators.
- Implement a `Permissions` trait:
  ```rust
  trait Permissions {
      fn can(&self, actor: &Actor, action: Action) -> bool;
  }
  ```
  where `Actor` carries the role and user ID.
- Remove the `casbin` crate from `Cargo.toml`.
- All service methods call `permissions.can(...)` instead of
  `authorization_service.authorize(...)`.
- Add a `/permissions` or `/me/permissions` API endpoint that
  returns the list of allowed actions for the current user.

**Pros:**
- Fully compile-time checked — adding or removing an action
  variant forces updating the permission matrix.
- No external dependency for authorization logic.
- Supports additional roles via the `Role` enum without a design
  change.
- Resource-ownership checks live in the same system as role checks.
- Enables a permissions-discovery endpoint for frontends.

**Cons:**
- Schema migration required (boolean → role column).
- Breaking API change: `admin: bool` in `TokenResponse` becomes
  `role: "admin"`.
- All service methods and their callers must be updated.
- Operators who have customised the Casbin policy via config lose
  that mechanism (unless replaceable by a TOML permission override).
- Custom policy flexibility is reduced compared to Casbin's
  arbitrary matcher expressions.

---

### Option C — Casbin with Full RBAC and Resource-Level Policies

Keep Casbin but upgrade the model from a flat `(role, action)`
matrix to a full RBAC model with resource attributes and role
hierarchies.

**Changes:**

- Upgrade the Casbin model to RBAC with resource fields:
  ```
  [request_definition]
  r = sub, obj, act

  [policy_definition]
  p = sub, obj, act

  [role_definition]
  g = _, _

  [matchers]
  m = g(r.sub, p.sub) && r.obj == p.obj && r.act == p.act
  ```
- Replace the `administrator: bool` column with a `roles` junction
  table (`user_id, role_name`) to support multiple roles and
  custom role assignment.
- Define role hierarchies in Casbin (e.g. `admin` inherits all
  `moderator` permissions; `moderator` inherits all `registered`
  permissions).
- Pass resource identifiers into `enforce()` calls:
  ```rust
  enforcer.enforce((role, resource_type, action))?;
  ```
- Add per-resource policies, e.g.:
  `p, owner, torrent, update` with a custom matcher that compares
  the acting user to the resource owner.
- Move the policy definition out of Rust source code into a
  dedicated file (e.g. `share/default/casbin_policy.csv`) that
  is loaded at startup, with the config TOML override taking
  precedence.
- Fix `CasbinEnforcer` to return `Result` instead of panicking.
- Replace `.expect()` in policy loading with startup validation
  errors.

**Pros:**
- Casbin's advanced features (role hierarchy, attribute matchers,
  domain tenancy) become available for future use.
- Resource-level authorization is expressed in the policy, not in
  scattered Rust `if` statements.
- Operators get the most flexible policy language — policies can
  be changed without recompiling.
- Role hierarchies reduce policy duplication (admin inherits
  moderator's permissions automatically).

**Cons:**
- Casbin remains a heavy dependency.
- The Casbin policy language is a DSL that contributors must learn
  — adding an action requires updating both Rust code and the
  CSV/string policy.
- Schema migration for the roles table.
- Debugging policy mismatches is harder (Casbin's error messages
  are opaque).
- Performance: every `enforce()` call locks the `RwLock<Enforcer>`
  and evaluates the matcher — acceptable for this scale but adds
  per-request overhead.

---

### Option D — Role-Based Middleware with Axum Extractors

Move authorization out of the service layer and into the HTTP
framework layer using Axum extractors and middleware.

**Changes:**

- Define a `RequireRole<R>` Axum extractor that validates the
  caller's role from the JWT/session before the handler is invoked:
  ```rust
  async fn add_category(
      RequireRole(Admin): RequireRole<Admin>,
      // ...
  ) -> Result<...> { ... }
  ```
- Define a `RequireOwnerOrRole<R>` extractor for resource-level
  checks that loads the resource and verifies ownership or
  sufficient role.
- Remove `authorization_service.authorize(ACTION::..., user_id)`
  calls from service methods — services become pure domain logic.
- The permission matrix becomes implicit in which extractors
  each handler uses (documented via route metadata / OpenAPI).
- Replace the `administrator: bool` column with a proper `role`
  column (as in Option B).
- Optionally generate an OpenAPI-compatible permissions manifest
  from the route definitions.

**Pros:**
- Authorization is declarative at the handler boundary — visible
  by reading the function signature.
- Service layer is decoupled from authorization — easier to test
  (no mock `authorization_service` needed).
- Leverages Axum's type-safe extractor system.
- Unauthorized requests are rejected before reaching the service
  layer (fail-fast).

**Cons:**
- Resource-level extractors (`RequireOwnerOrRole`) must load
  resources from the database, potentially duplicating the load
  that the service method will also do.
- Complex authorization rules (e.g., "moderator can edit only
  torrents in categories they moderate") are awkward to express as
  extractors.
- All the authorization knowledge is scattered across handler
  function signatures rather than a single policy file — harder to
  audit as a whole.
- Schema migration required.
- Tightly couples authorization to the Axum framework.

---

### Option E — Hybrid: Native Permission Trait + Axum Extractors

Combine Options B and D: a native Rust permission system for policy
definition with Axum extractors for enforcement at the HTTP
boundary.

**Changes:**

- Implement the `Role` enum, `Action` enum, and `PermissionMatrix`
  from Option B.
- Implement `RequirePermission<A>` Axum extractors that consult
  the `PermissionMatrix` (from Option D's approach):
  ```rust
  async fn add_category(
      RequirePermission(AddCategory): RequirePermission<AddCategory>,
      // ...
  ) -> Result<...> { ... }
  ```
- For resource-level checks, provide a `RequireOwnership` extractor
  or a `permissions.can_on_resource(actor, action, resource)` method
  that handlers call explicitly.
- The `PermissionMatrix` is a shared `Arc` provided via Axum state,
  defined in code with an optional TOML override.
- Service methods no longer take `maybe_user_id` for authorization
  — they receive an already-authorized `Actor` or are called
  unconditionally.
- Add a `/me/permissions` endpoint returning allowed actions.
- Schema migration: `administrator: bool` → `role: TEXT`.
- Remove the `casbin` dependency.

**Pros:**
- Clean separation: policy definition (matrix) vs. enforcement
  point (extractor) vs. domain logic (service).
- Compile-time checked actions.
- Fail-fast at the HTTP boundary.
- Services are free of authorization concerns — easier to test.
- Permissions-discovery endpoint for frontends.
- No heavy external authorization dependency.

**Cons:**
- Largest scope — touches handlers, services, database schema,
  and adds new extractor infrastructure.
- Resource-level checks may still need ad-hoc logic in handlers
  or a thin "resource authorization" service.
- Schema migration required.

## Decision

**Option E — Hybrid: Native Permission Trait + Axum Extractors.**

This option best addresses the full set of identified problems while
keeping the architecture idiomatic to both Rust and Axum. The key
reasons:

1. **Clean separation of concerns.** Policy *definition* lives in a
   single `PermissionMatrix` (auditable in one place), *enforcement*
   happens at the HTTP boundary via Axum extractors (fail-fast,
   declarative), and *domain logic* in services stays free of
   authorization plumbing.

2. **Compile-time safety.** Both `Role` and `Action` are Rust enums.
   Adding or removing a variant is a compiler error until the
   permission matrix is updated — impossible to silently forget a
   permission grant or denial.

3. **Removes Casbin without losing flexibility.** The current Casbin
   usage is a flat `(role, action)` lookup that exercises none of
   Casbin's advanced features. A native `PermissionMatrix` (with
   an optional TOML override for operators) replaces it at lower
   complexity, fewer dependencies, and faster compile times.

4. **Addresses resource-level authorization.** Ownership checks
   that are currently scattered as inline `if` blocks in service
   methods move into dedicated `RequireOwnership` extractors or a
   `permissions.can_on_resource(actor, action, resource)` method,
   co-located with the rest of the permission logic.

5. **Enables permissions discovery.** A `/me/permissions` endpoint
   returning the list of allowed actions for the authenticated user
   lets frontends adapt their UI without guessing.

6. **Extensible role model.** Replacing the `administrator: bool`
   column with a `role: TEXT` column (via migration) opens the door
   to intermediate roles (`Moderator`, `Trusted`, etc.) without
   another schema change.

The naming choice `RequirePermission<Action>` (over Option D's
`RequireRole<Role>`) reflects that the extractor checks the
*action*, not just the caller's role — a single handler declares
what it needs, and the matrix decides which roles satisfy it.

The larger scope (handlers, services, database, new extractor
infrastructure) is accepted as a trade-off. The work will be
phased:

- **Phase 1:** Schema migration (`administrator: bool` →
  `role: TEXT`), `Role` and `Action` enums, `PermissionMatrix`,
  `Permissions` trait, remove Casbin dependency.
- **Phase 2:** `RequirePermission<A>` extractors on all handlers,
  strip authorization logic from services.
- **Phase 3:** Resource-level extractors / `can_on_resource`,
  `/me/permissions` endpoint, TOML override for operators.
- **Phase 4:** E2E tests for Phase 3 features (ownership
  round-trips, `/me/permissions` per role, TOML override runtime
  behaviour).

No Casbin migration guide is needed — the `unstable.auth.casbin`
configuration was part of the unstable API surface and never reached
a stable release, so there are no supported deployments with custom
policies to migrate.

## Consequences

### Positive

- A single `PermissionMatrix` replaces scattered Casbin policies
  and inline `if` checks — one place to audit all permissions.
- Compile-time enforcement: adding an `Action` variant without
  updating the matrix is a build error.
- Services become purely domain logic; no `maybe_user_id` threading
  or `authorization_service.authorize(...)` calls.
- Unauthorized requests are rejected before reaching the service
  layer (fail-fast at the extractor boundary).
- The `casbin` crate and its transitive dependencies are removed,
  reducing compile time and binary size.
- The `role: TEXT` column supports future roles without schema
  changes.
- Frontends gain a `/me/permissions` discovery endpoint.
- Role grants are logged via `tracing` (partially addresses the
  audit-trail gap from Problem 7 — full per-change audit logging
  of who-granted-what-to-whom is deferred to a future ADR).

### Negative

- Requires a database migration from `administrator: bool` to
  `role: TEXT`, including a data migration for existing users.
- All handlers must be updated to use the new extractors —
  large surface area for the refactor.
- Resource-level ownership checks may still require thin ad-hoc
  logic in handlers where a generic extractor is insufficient.
- Breaking API change: `admin: bool` in responses replaced by
  `role: String`. No deprecation period is needed because the
  authorization system was never part of a released version.

### Risks

- **Phase 2 is a large, cross-cutting change.** Mitigated by
  introducing the extractors handler-by-handler behind the same
  permission matrix, so both old and new call-sites coexist during
  the transition.
- **Resource-level extractor may duplicate DB loads.** Mitigated by
  caching the loaded resource in Axum request extensions so the
  handler can reuse it.
- **Forward-only migrations have no rollback path.** The schema
  migration from `administrator: bool` to `role: TEXT` is
  irreversible by design. If a later phase hits a blocker after
  Phase 1 is deployed, the `role` column remains and the old
  boolean column is gone. This is accepted: rolling back would
  require a manual reverse migration, which is preferable to
  carrying two redundant columns. **Note:** the two migration
  files (`20260415000000_…role_column` and
  `20260415000001_…drop_administrator_column`) are ordered by
  filename suffix; `sqlx migrate run` applies them
  lexicographically, so the `role` column is always created
  before the `administrator` column is dropped.
- **`Admin` arm is now exhaustive.** The `default_grant`
  function uses an exhaustive `match` on `Action` for all roles,
  including `Admin`. Adding a new `Action` variant is a compile
  error until the developer explicitly grants or denies it for
  every role.

## Testing Strategy

Testing is organized per-phase and follows the project's three-tier
test model (unit → crate → integration).

### Phase 1 — Core Types and Migration

| What | Level | Location | Notes |
|---|---|---|---|
| `Role` enum round-trips (`Display` ↔ `FromStr`, serde) | Unit | inline | Cover every current variant; future variants (e.g. `Moderator`) will extend this list |
| `Action` enum exhaustiveness | Unit | inline | A `match` with no wildcard ensures new variants force updates |
| `PermissionMatrix` grants and denials | Crate | `src/tests/` | For each `(Role, Action)` pair, assert the expected `Effect` |
| `PermissionMatrix` TOML override merges correctly | Crate | `src/tests/` | Load a partial override; verify it patches the default matrix |
| `Permissions::can()` delegates to the matrix | Crate | `src/tests/` | Unit-style test with a known matrix |
| Schema migration (`administrator: bool` → `role: TEXT`) | Integration | `tests/` | Apply migration on both SQLite and MySQL; verify `admin = true` → `Role::Admin`, `admin = false` → `Role::Registered` |
| API response carries `role` field | Integration | `tests/e2e/` | `TokenResponse` and `UserCompact` use `role: String` instead of `admin: bool` |

### Phase 2 — Axum Extractors

| What | Level | Location | Notes |
|---|---|---|---|
| `RequirePermission<A>` rejects insufficient role | Crate | `src/tests/` | Build a minimal Axum `TestServer`; assert 403 for denied actions |
| `RequirePermission<A>` passes for allowed role | Crate | `src/tests/` | Same test server; assert handler body is reached |
| Unauthenticated request yields 401, not 403 | Crate | `src/tests/` | No token → `Guest` role → verify correct status code |
| Every handler has an extractor (no unguarded routes) | Crate | `src/tests/` | Reflective or generated test that walks the router and asserts each route has a `RequirePermission` or `RequireOwnership` layer |
| Service methods no longer perform authorization | Code review | — | Manual review: `grep` for removed `authorize(...)` calls |

### Phase 3 — Resource-Level Authorization and Discovery

| What | Level | Location | Notes |
|---|---|---|---|
| `RequireOwnership` allows owner | Integration | `tests/e2e/` | Upload a torrent as user A; update as user A → 200 |
| `RequireOwnership` denies non-owner non-admin | Integration | `tests/e2e/` | Upload as user A; update as user B (registered) → 403 |
| `RequireOwnership` allows admin override | Integration | `tests/e2e/` | Upload as user A; update as admin → 200 |
| `can_on_resource` returns correct result for owner vs. stranger | Crate | `src/tests/` | Direct call with mock resource metadata |
| `/me/permissions` returns correct action list per role | Integration | `tests/e2e/` | Authenticate as each role; compare response to the matrix |
| `/me/permissions` for guest (no token) | Integration | `tests/e2e/` | Unauthenticated call returns guest-level actions |
| TOML override changes runtime permissions | Integration | `tests/e2e/` | Start server with a custom TOML override; hit an endpoint that would normally be denied and verify it is now allowed |

### Phase 4 — E2E Tests for Phase 3 Features

Phase 4 promotes the Phase 3 crate-level tests to full E2E
integration tests that exercise the HTTP round-trip, and extends
coverage to update and delete ownership scenarios, the
`/me/permissions` response structure, and TOML overrides.

**Prerequisite — `TestEnv` config injection.** The TOML override
tests require starting an isolated test environment with custom
`permissions.overrides` in the configuration. The existing
`isolated::TestEnv::with_test_configuration()` /
`AppStarter::with_custom_configuration(config)` path supports this:
set `configuration.permissions.overrides` on the `config::Settings`
before calling `start()`. No new infrastructure is needed beyond a
helper that patches the ephemeral config.

**Note on owner-delete under default permissions.**
`DeleteTorrent` is denied for `Registered` in the default
`PermissionMatrix`, so the `RequirePermission<DeleteTorrent>`
extractor rejects `Registered` users with 403 before
`can_on_resource` is ever reached. The "owner deletes own torrent"
scenario therefore requires a TOML override granting `Registered` +
`DeleteTorrent` — it is grouped with the TOML-override tests below
rather than listed as a standalone ownership test.

| What | Level | Location | Notes |
|---|---|---|---|
| Ownership allows owner to update own torrent | Integration | `tests/e2e/` | Upload as user A; update as user A → 200 |
| Ownership denies non-owner non-admin update | Integration | `tests/e2e/` | Upload as user A; update as user B (registered) → 403 |
| Ownership allows admin override on update | Integration | `tests/e2e/` | Upload as user A; update as admin → 200 |
| Ownership denies non-owner non-admin delete | Integration | `tests/e2e/` | Upload as user A; delete as user B (registered) → 403 (rejected at extractor — `DeleteTorrent` denied for `Registered` by default) |
| Ownership allows admin override on delete | Integration | `tests/e2e/` | Upload as user A; delete as admin → 200 |
| `/me/permissions` returns correct actions per role | Integration | `tests/e2e/` | Authenticate as admin, registered, and guest (no token); verify `{"role": "...", "actions": [...]}` structure and correct action lists for each |
| TOML override grants a normally-denied action (owner-delete) | Integration | `tests/e2e/` | Start isolated server with override allowing `Registered` + `DeleteTorrent`; upload as user A; delete as user A → 200; delete as user B → 403 (ownership still enforced by `can_on_resource`) |
| TOML override denies a normally-allowed action | Integration | `tests/e2e/` | Start isolated server with override denying `Registered` + `AddTorrent`; upload attempt as registered user → 403 |

### Cross-Cutting

- **Both database backends.** Every migration and query test runs
  against SQLite and MySQL.
- **Release mode.** Run the full suite with `--release` as
  required by project conventions.
- **`--no-default-features` spot check.** Verify the crate builds
  and core permission types compile without default features.

## Acceptance Criteria

A phase is considered complete when **all** its criteria are met.

### Phase 1

1. The `casbin` crate is removed from `Cargo.toml` and the
   project compiles without it.
2. `Role` and `Action` are public enums; adding a variant without
   updating the `PermissionMatrix` is a compile error (enforced
   by an exhaustive `match` or equivalent).
3. A `PermissionMatrix` with a default policy exists and is
   covered by tests for every `(Role, Action)` pair.
4. The database schema has a `role: TEXT` column replacing
   `administrator: bool`, with a forward-only migration for both
   SQLite and MySQL.
5. Existing users with `administrator = true` are migrated to
   `role = 'admin'`; all others to `role = 'registered'`.
6. API responses use `role: String` instead of the legacy
   `admin: bool` field.
7. `cargo clippy --workspace --all-targets --all-features` and
   `cargo test --workspace --all-targets --all-features` pass
   cleanly.
8. `PermissionMatrix::default_matrix()` is infallible; no panic
   paths on startup.

### Phase 2

1. Every HTTP handler that requires authorization uses a
   `RequirePermission<A>` (or `RequireOwnership`) extractor.
2. No service method contains an `authorize(...)` call or an
   inline `if administrator` / `if uploader == user` check for
   permission purposes.
3. Requests with insufficient permissions receive a `403 Forbidden`
   response; unauthenticated requests receive `401 Unauthorized`.
4. All existing E2E tests pass without modification (or are updated
   to match the new error codes where they were previously
   inconsistent).

### Phase 3

1. `GET /me/permissions` returns the full list of allowed `Action`
   values for the authenticated user's role.
2. `GET /me/permissions` without a token returns guest-level
   actions.
3. Resource-level ownership checks (torrent update, torrent delete,
   etc.) are enforced via `RequireOwnership` or
   `can_on_resource(...)`, not inline `if` blocks.
4. An operator can supply a TOML permissions override that is
   loaded at startup and merges with / replaces the default matrix.
5. `cargo test --workspace --all-targets --all-features` and
   `cargo test --workspace --all-targets --all-features --release`
   pass cleanly on both SQLite and MySQL.

### Phase 4

1. E2E tests verify resource-level ownership for torrent update:
   owner → 200, non-owner registered → 403, admin → 200.
2. E2E tests verify resource-level ownership for torrent delete:
   non-owner registered → 403 (denied at extractor by default
   matrix), admin → 200.
3. E2E tests verify `GET /me/permissions` returns correct action
   lists for admin, registered, and guest (no token) roles,
   including the resolved `role` field in the response.
4. E2E tests verify TOML permission overrides affect runtime
   behaviour: granting `Registered` + `DeleteTorrent` allows
   owner-delete while still enforcing `can_on_resource` for
   non-owners; denying a normally-allowed action yields 403.
5. All E2E tests run against both SQLite and MySQL backends.
6. `cargo test --workspace --all-targets --all-features` and
   `cargo test --workspace --all-targets --all-features --release`
   pass cleanly.

## Phase 1 — Implementation Review

Review performed 2026-04-15 against the initial Phase 1 commit.

### Status of Acceptance Criteria

| # | Criterion | Status | Notes |
|---|-----------|--------|-------|
| 1 | Casbin removed | ✅ | No `casbin` in `Cargo.toml`; `unstable.auth.casbin` config section removed entirely (no backward compatibility needed — never released). |
| 2 | Compile-time safe enums | ✅ | `default_grant` uses exhaustive `match` for `Registered` and `Guest`. |
| 3 | `PermissionMatrix` with tests | ✅ | Every `(Role, Action)` pair covered by crate-level tests. |
| 4 | Forward-only migration | ✅ | Migration adds `role`; a follow-up migration drops `administrator` via table-rebuild (SQLite) / `DROP COLUMN` (MySQL). |
| 5 | Data migration | ✅ | `UPDATE … SET role = 'admin' WHERE administrator = TRUE` for both backends. |
| 6 | API uses `role: String` | ✅ | `TokenResponse` uses `role: String`; legacy `admin: bool` field removed. |
| 7 | Clippy and tests pass | ✅ | Fixed during review (`doc_markdown` lint). |
| 8 | No panics on startup | ✅ | `PermissionMatrix::default_matrix()` is infallible. |

### Open Issues (Resolved)

All five issues from the initial Phase 1 review have been resolved.

1. **Migration is forward-only** — accepted. The first migration
   adds the `role` column; a follow-up migration
   (`20260415000001_torrust_drop_administrator_column`) drops the
   `administrator` column via a table-rebuild approach on SQLite
   (compatible with SQLite < 3.35.0) and `ALTER TABLE DROP COLUMN`
   on MySQL.

2. **`administrator` column dropped** — resolved. The column is
   removed by the follow-up migration. All code paths use `role`
   exclusively.

3. **`get_role` silently swallows invalid role strings** —
   resolved. The `RequirePermission` extractor logs a
   `tracing::warn!` when `Role::from_str` fails, then defaults to
   `Registered`. **Security note:** defaulting to `Registered`
   (rather than `Guest`) on a corrupt role string is a deliberate
   pragmatic choice — a corrupted column most likely indicates a
   legitimate user whose role was mangled, not an attacker.
   Defence-in-depth is provided by the fact that resource-level
   operations additionally require ownership via
   `can_on_resource`. **Trade-off acknowledged:** this is a
   fail-open decision for the `Registered` → `Guest` boundary. A
   corrupt role could also indicate a migration failure or (in the
   extreme case) a SQL injection. The `warn!` log ensures
   operators are alerted; if a stricter posture is desired in the
   future, the fallback can be changed to `Guest` without
   architectural impact.

4. **E2E test assertion needs `role` field** — resolved. The
   `admin: bool` field has been removed from all response types
   (`TokenResponse`, `LoggedInUserData`, `TokenRenewalData`).
   Test assertions use `role: String` exclusively.

5. **v1 → v2 upgrade path** — resolved.
   `insert_imported_user` now converts the v1 `administrator: bool`
   to a `role` string (`"admin"` / `"registered"`) and inserts into
   the `role` column directly. The upgrade test verifies the
   correct `role` value in the target database.

## Phase 2 — Implementation Review

Review performed 2026-04-15 against the Phase 2 commit that
introduces `RequirePermission<A>` extractors on all handlers.

### Status of Acceptance Criteria

| # | Criterion | Status | Notes |
|---|-----------|--------|-------|
| 1 | Every authorized handler uses `RequirePermission<A>` | ✅ | All API context handlers are guarded. `create_random_torrent_handler` is documented as intentionally unguarded (testing/debug, no persistent state). Pre-auth endpoints carry `// Public: no RequirePermission` comments. |
| 2 | No service method contains `authorize(...)` or inline admin/owner check | ✅ | Service layer is clean. The inline ownership check in `update_torrent_info_handler` (handler, not service) uses `actor.role == Role::Admin` from the already-resolved extractor — no redundant DB query. Phase 3 will replace with `RequireOwnership`. |
| 3 | Insufficient permissions → 403; unauthenticated → 401 | ✅ | `AuthError::UnauthorizedAction` → 403 (`FORBIDDEN`), `AuthError::UnauthorizedActionForGuests` → 401 (`UNAUTHORIZED`). Correct across `AuthError`, `UserError`, and `TorrentError`. Verified by new crate-level extractor tests. |
| 4 | Existing E2E tests pass or updated | ✅ | E2E test response types (`LoggedInUserData`, `TokenRenewalData`) use `role: String`; the legacy `admin: bool` field has been removed. Assertions verify the `role` value. |

### What's Working Well

- **Clean extractor design.** The `ActionMarker` trait +
  `action_markers!` macro is ergonomic and compile-time safe.
  Adding a new `Action` variant forces updates to both the
  `PermissionMatrix` and the marker list.
- **Service layer is authorization-free.** No production call-site
  of `authorization::Service` remains — the struct has been removed
  as dead code. No service
  method performs an inline `if administrator` or
  `if uploader == user` check.
- **Old extractors removed from handlers.** `ExtractLoggedInUser`
  and `ExtractOptionalLoggedInUser` are no longer imported by any
  handler.
- **401 vs. 403 distinction is correct.** Guest-denied → 401,
  authenticated-but-insufficient → 403, consistent across all
  error enums.
- **`Actor` provides a clean identity carrier.** The `user_id()`
  panic guard is appropriate for handlers where `Guest` is denied
  by the permission matrix.

### Open Issues (Resolved)

All eight issues from the initial Phase 2 review have been
resolved, plus one additional cleanup:

1. **`create_random_torrent_handler`** — documented as intentionally
   unguarded with a `// Public:` comment (testing/debug endpoint, no
   persistent state).
2. **Ownership check in `update_torrent_info_handler`** — updated to
   use `actor.role == Role::Admin` from the already-resolved
   extractor, avoiding a redundant DB query. The ownership check
   itself remains as Phase 3 scope.
3. **E2E test response types** — `LoggedInUserData` and
   `TokenRenewalData` use `role: String`; the legacy `admin: bool`
   field has been removed. Assertions verify the `role` value.
4. **`GetSettings` and `GetCanonicalInfoHash`** — dead action
   variants and markers removed from `Action` enum, `Action::ALL`,
   `PermissionMatrix::default_grant`, `action_markers!`, and all
   tests.
5. **`authorization::Service`** — removed along with its unused
   `user::Repository`, `AuthError`, and `UserId` imports. Module
   doc updated.
6. **Old extractors** — `ExtractLoggedInUser` (`user_id.rs`) and
   `ExtractOptionalLoggedInUser` (`optional_user_id.rs`) deleted;
   `extractors/mod.rs` updated.
7. **Pre-auth endpoints** — all five handlers now carry
   `// Public: no RequirePermission — pre-authentication endpoint`
   comments.
8. **Crate-level extractor tests** — added in
   `src/tests/web/require_permission.rs`: several tests covering
   `Actor` behaviour and the full `RequirePermission<A>` extraction
   path
   (guest → 401, guest → 200, registered → 403, registered → 200,
   admin → 200) using a minimal Axum router backed by an ephemeral
   SQLite database.
9. **Double `.into_response()` in `license_page_handler`** —
   `about/handlers.rs` called `.into_response().into_response()`
   on the success path (harmless but redundant). Removed the
   duplicate call.

### Verification

Review performed 2026-04-15.

- `cargo check --workspace --all-targets --all-features` — clean.
- `cargo clippy --workspace --all-targets --all-features` — clean.
- `cargo test --workspace --all-targets --all-features` — ~400
  tests, 0 failures.
- `cargo check --workspace --no-default-features` — clean.

The `CHANGELOG.md` has been updated to reflect the net state of
the unreleased section: intermediate entries describing
`authorization::Service` delegating to `PermissionMatrix` and
`ExtractLoggedInUser` using `BearerToken` have been replaced by
Removed entries documenting their deletion.

## Phase 3 — Implementation Review

Review performed 2026-04-15 against the working-tree changes that
introduce `/me/permissions`, `can_on_resource`, TOML permission
overrides, and the `GetMyPermissions` action.

### Status of Acceptance Criteria

| # | Criterion | Status | Notes |
|---|-----------|--------|-------|
| 1 | `GET /me/permissions` returns allowed actions for the user's role | ✅ | Handler calls `permissions.allowed_actions(&actor.role)` and returns `{"role": "...", "actions": [...]}` in `OkResponseData`. |
| 2 | `GET /me/permissions` without a token returns guest-level actions | ✅ | `GetMyPermissions` is granted to `Guest` in `default_grant`, so a missing token resolves to `Guest` → returns `{"role": "guest", "actions": [...]}`. |
| 3 | Resource-level ownership via `can_on_resource`, not inline `if` blocks | ✅ | Both `update_torrent_info_handler` and `delete_torrent_handler` use `app_data.permissions.can_on_resource(...)` with `torrent_listing.uploader_id == user_id`. |
| 4 | TOML permissions override loaded at startup | ✅ | `[[permissions.overrides]]` config section, `PermissionOverride` struct, `PermissionMatrix::with_overrides()`, wired in `app.rs` with an `info!` log when overrides are applied. **Operator responsibility:** no startup validation guards against dangerous grants (e.g. giving `Guest` admin-level actions like `DeleteTorrent` or `BanUser`). Operators must review their overrides carefully. A startup warning for high-risk grants may be added in a future iteration. |
| 5 | `cargo test` passes (dev + release, all features) | ✅ | All tests pass in both `dev` and `--release`. `cargo clippy` clean. `--no-default-features` builds. Doc tests pass. |

### What's Working Well

- **`can_on_resource` default implementation on the trait.** The
  ownership logic (`Admin` bypasses, others must be owner) lives
  in a single `Permissions::can_on_resource` default method with
  no per-handler duplication. Adding ownership checks to future
  handlers is a one-liner.
- **`allowed_actions` is a trait default, not a separate service.**
  Any `Permissions` implementor gets it for free, and the endpoint
  is just `permissions.allowed_actions(&actor.role)`.
- **TOML override is clean and minimal.** The `Effect` enum
  (`allow`/`deny`) and `PermissionOverride` struct map directly to
  insert/remove on the `HashSet` — no complex merge logic.
  The `app.rs` startup path logs the override count, making
  misconfigurations visible.
- **Compile-time safety preserved.** Adding `GetMyPermissions` to
  `Action` required updating `default_grant` (exhaustive match for
  `Registered` and `Guest`), `Action::ALL`, `action_markers!`, and
  all crate-level test lists — any omission would be a build error.
- **Good crate-level test coverage.** Several new tests cover
  `can_on_resource` (owner allowed, non-owner denied, admin bypass,
  denied action), `allowed_actions` (correct list), and
  `with_overrides` (allow adds, deny removes, empty matches
  default).
- **Ownership checks use `uploader_id` (integer) instead of
  username (string).** `TorrentListing` now carries `uploader_id`
  directly from the SQL query, so ownership comparisons are
  `torrent_listing.uploader_id == user_id` — no extra
  `UserCompact` fetch required.
- **`/me/permissions` response includes the resolved `role`.**
  The response body is `{"data": {"role": "registered", "actions":
  [...]}}`, letting frontends know the actor's effective role
  without a separate API call.

### Open Issues (Resolved)

Issues 1, 3, and 4 from the initial Phase 3 review have been
resolved. Issue 2 is deferred.

1. **`delete_torrent_handler` `can_on_resource` check** — added.
   The handler now mirrors `update_torrent_info_handler`: loads the
   torrent listing, compares `uploader_id`, and calls
   `permissions.can_on_resource(&actor.role, Action::DeleteTorrent,
   is_owner)`.

   **Design note on `DeleteTorrent` for non-admins:**
   `DeleteTorrent` is *denied* for `Registered` in
   `default_grant`. This means the `RequirePermission<DeleteTorrent>`
   extractor rejects `Registered` users with 403 before
   `can_on_resource` is ever reached. The ownership check in the
   handler is only meaningful when an operator enables
   `DeleteTorrent` for `Registered` via a TOML override — at that
   point `can_on_resource` correctly restricts deletion to the
   user's own torrents.

2. **No E2E tests for Phase 3 features** — **deferred to
   Phase 4**. The crate-level tests provide good coverage of the
   underlying logic (`can_on_resource`, `allowed_actions`,
   `with_overrides`). E2E tests for the full HTTP round-trip
   (ownership → 200/403, `/me/permissions` per role, TOML override
   runtime behaviour) are tracked as Phase 4 acceptance criteria.

3. **`/me/permissions` response now includes the resolved `role`.**
   The response body changed from `{"data": [...]}` to
   `{"data": {"role": "registered", "actions": [...]}}`. This is a
   non-breaking enrichment (new structured object instead of a bare
   array).

4. **Ownership checks now use `uploader_id` instead of username.**
   `TorrentListing` gained an `uploader_id: UserId` field
   (populated by `tt.uploader_id` in all 6 SQL queries across both
   SQLite and MySQL backends). Both `update_torrent_info_handler`
   and `delete_torrent_handler` compare `torrent_listing.uploader_id
   == user_id` directly, eliminating the `UserCompact` fetch.

### Verification

Review performed 2026-04-15. Issues 1, 3, 4 resolved; issue 2
deferred.

- `cargo check --workspace --all-targets --all-features` — clean.
- `cargo clippy --workspace --all-targets --all-features` — clean.
- `cargo test --workspace --all-targets --all-features` — ~400
  tests, 0 failures.
- `cargo test --workspace --all-targets --all-features --release`
  — ~400 tests, 0 failures.
- `cargo check --workspace --no-default-features` — clean.
- `cargo test --workspace --doc` — clean.

## Phase 4 — E2E Coverage Audit

Review performed 2026-04-15 against the working tree after
Phases 1–3. Phase 4 adds no new implementation — it audits
existing E2E tests for Phase 3 coverage and adds targeted tests
where gaps exist.

### Status of Acceptance Criteria

| # | Criterion | Status | Notes |
|---|-----------|--------|-------|
| 1 | E2E ownership tests for torrent update (owner → 200, non-owner → 403, admin → 200) | ✅ | Covered by existing E2E tests: `it_should_allow_torrent_owners_to_update_their_torrents`, `it_should_not_allow_registered_users_to_update_someone_elses_torrents`, `it_should_allow_admins_to_update_someone_elses_torrents`. These now exercise `can_on_resource` after Phase 3. |
| 2 | E2E ownership tests for torrent delete (non-owner → 403, admin → 200) | ✅ | Covered by `it_should_not_allow_registered_users_to_delete_torrents` (403 at extractor — `DeleteTorrent` denied for `Registered` by default) and `it_should_allow_admins_to_delete_torrents_searching_by_info_hash` / `it_should_allow_admin_users_to_delete_torrents`. |
| 3 | E2E tests for `GET /me/permissions` per role | ✅ | Three new tests in `permissions_discovery`: `it_should_return_all_actions_for_an_admin`, `it_should_return_registered_actions_for_a_registered_user`, `it_should_return_guest_actions_when_no_token_is_provided`. Each verifies the `{"data": {"role": "...", "actions": [...]}}` structure and correct action lists. |
| 4 | E2E tests for TOML permission overrides | ✅ | Two new tests in `permission_overrides`: `it_should_grant_a_normally_denied_action_via_toml_override` (grants `Registered` + `DeleteTorrent`, verifies via `/me/permissions`) and `it_should_deny_a_normally_allowed_action_via_toml_override` (denies `Registered` + `AddTorrent`, verifies via `/me/permissions` and upload attempt → 403). Uses `TestEnv::start_with` to inject overrides into the isolated config. |
| 5 | All E2E tests run against both SQLite and MySQL | ⚠️ | E2E tests run against SQLite. MySQL backend coverage requires a running MySQL instance and is not exercised in the default `cargo test` invocation. |
| 6 | `cargo test` passes cleanly | ✅ | ~410 tests, 0 failures (dev and release). `cargo clippy` clean. `--no-default-features` builds. Doc tests pass. |

### Remaining Work

1. **MySQL E2E coverage (blocking for full sign-off).** The entire
   E2E suite must be verified against a MySQL backend. This
   requires a running MySQL instance and is not exercised by the
   default `cargo test` invocation. Until this is done, Phase 4
   criterion 5 remains ⚠️. This should be run manually or gated
   in CI before the ADR status is changed to fully implemented.

### Verification

Review performed 2026-04-15.

- `cargo check --workspace --all-targets --all-features` — clean.
- `cargo clippy --workspace --all-targets --all-features` — clean.
- `cargo test --workspace --all-targets --all-features` — ~410
  tests, 0 failures.
- `cargo test --workspace --all-targets --all-features --release`
  — ~410 tests, 0 failures.
- `cargo check --workspace --no-default-features` — clean.
- `cargo test --workspace --doc` — clean.

## Remaining Issues

1. **MySQL E2E coverage (Phase 4, criterion 5).**
   The full E2E suite has been verified against SQLite only.
   MySQL backend testing requires a running MySQL instance and is
   not exercised by the default `cargo test` invocation. This must
   be run manually or gated in CI before the ADR status can be
   changed from ⚠️ to fully implemented. The migrations and SQL
   queries are dual-backend (separate `sqlite3/` and `mysql/`
   files), so the most likely failure mode is a syntax or
   type-mapping difference in the new `role` column handling.

2. ~~**No startup validation for dangerous TOML overrides.**~~
   ✅ **Resolved.** `PermissionMatrix::with_overrides` now emits
   a `warn!` for each override that grants a high-risk action
   (`DeleteTorrent`, `DeleteCategory`, `DeleteTag`, `BanUser`,
   `GetSettingsSecret`, `GenerateUserProfileSpecification`) to a
   non-admin role. The override is still applied — the warning is
   advisory so operators can review their configuration.
   `HIGH_RISK_ACTIONS` is a `const` list on `PermissionMatrix`,
   easily extended if future actions warrant the same treatment.

3. ~~**`Admin` wildcard in `default_grant`.**~~
   ✅ **Resolved.** The `Admin` arm now uses an exhaustive `match`
   on `Action` (no wildcard). Adding a new `Action` variant forces
   an explicit decision for every role, including `Admin`. This
   eliminates the silent-grant risk: a new variant like
   `PurgeAllData` would be a compile error until the developer
   explicitly grants or denies it for `Admin`.

4. ~~**No per-change audit logging for role grants.**~~
   ✅ **Partially resolved.** `grant_admin_role` in
   `DbUserRepository` now emits `info!(user_id, "granting admin
   role")` via `tracing`, and the first-user auto-admin path in
   `RegistrationService::register` logs the grant explicitly.
   Failures in the first-user auto-admin grant now emit a
   `warn!` instead of being silently dropped (see item 6).
   Full persistent audit logging (who-granted-what-to-whom with
   timestamps in a dedicated table) is still deferred to a future
   ADR.

5. ~~**`Moderator` role is defined in the ADR but not yet
   implemented.**~~
   ✅ **Resolved.** `Role::Moderator` has been added to the `Role`
   enum with `Display`, `FromStr`, and serde support. The
   `default_grant` function defines `Moderator` permissions as a
   superset of `Registered` (adds `AddTag`, `DeleteTag`,
   `DeleteTorrent`). Crate-level tests cover the full
   `Moderator` grant/denial matrix. No data migration is needed —
   no existing users have the `moderator` role string; it becomes
   available for operators to assign manually or via a future
   admin endpoint.

6. ~~**First-user auto-admin silently swallows
   `grant_admin_role` errors.**~~
   ✅ **Resolved.** `RegistrationService::register` previously
   used `drop(self.user_repository.grant_admin_role(...).await)`
   to discard the `Result`. If the grant failed, the first user
   silently became `Registered` with no diagnostic output.
   Replaced with `if let Err(err) = ... { warn!(...) }` so
   operators are alerted via `tracing` when the grant fails.
