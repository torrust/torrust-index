# ADR-T-007: Refactor the JWT System

**Status:** Implemented
**Date:** 2026-04-14

## Context

The JWT (JSON Web Token) authentication system has grown organically
and currently exhibits several structural and security problems.
This ADR catalogues the issues and presents options for a
comprehensive refactor.

### Current Architecture

JWT handling is spread across four locations with two distinct
claim types:

| Location | Purpose | Claim type |
|---|---|---|
| `services::authentication::JsonWebToken` | Sign/verify session tokens | `UserClaims` |
| `web::api::server::v1::auth::Authentication` | Thin wrapper delegating to `JsonWebToken` | `UserClaims` |
| `mailer::Service::get_verification_url` | Sign email-verification tokens | `VerifyClaims` |
| `services::user::RegistrationService::verify_email` | Verify email-verification tokens | `VerifyClaims` |

Both token types share the same HMAC-HS256 signing secret
(`auth.user_claim_token_pepper`).

### Identified Problems

1. **Single shared secret for all token purposes.**
   Session JWTs and email-verification JWTs share the same
   HMAC secret (`user_claim_token_pepper`). A leaked session
   token secret also compromises email-verification tokens,
   and vice-versa. There is no audience/purpose separation at
   the key level.

2. **HMAC-HS256 with a text "pepper" as the key.**
   The signing key is an arbitrary human-readable string
   (default: `"MaxVerstappenWC2021"`), used directly via
   `EncodingKey::from_secret(key.as_bytes())`. There is no
   minimum entropy requirement, no key derivation, and no
   support for asymmetric algorithms. HMAC-HS256 with a
   low-entropy secret is vulnerable to offline brute-force
   attacks on captured tokens.

3. **No `iss`, `aud`, or `sub` claims on session tokens.**
   `UserClaims` contains only `{ user: UserCompact, exp: u64 }`.
   It has no `iss` (issuer), `aud` (audience), `sub` (subject),
   or `iat` (issued-at) fields. RFC 7519 recommends these
   registered claims for interoperability and security. By
   contrast, `VerifyClaims` in the mailer *does* set `iss` and
   `sub`, but not `aud`.

4. **User data embedded verbatim in the JWT payload.**
   `UserClaims` embeds the full `UserCompact` struct
   (`user_id`, `username`, `administrator`) in the payload.
   This means the `administrator` flag is trusted from the token
   rather than re-checked from the database at each request. If
   a user's role changes, existing tokens carry stale privileges
   until they expire. Role escalation tokens remain valid for
   the full two-week window.

5. **Hard-coded expiration durations.**
   Session tokens expire in `1_209_600` seconds (2 weeks) with a
   `// todo` comment acknowledging this should be configurable.
   Email-verification tokens expire in `315_569_260` seconds
   (~10 years). The renewal threshold is a hard-coded
   `ONE_WEEK_IN_SECONDS`. None of these are configurable.

6. **Redundant / manual expiration checking.**
   `JsonWebToken::verify` passes `Validation::new(Algorithm::HS256)`
   to `jsonwebtoken::decode`, which already validates `exp` by
   default, yet the code performs an *additional* manual
   `if token_data.claims.exp < clock::now()` check. These two
   checks may disagree in edge cases (clock skew handling
   differs).

7. **`.unwrap()` when signing tokens.**
   `mailer::get_verification_url` calls `encode(...).unwrap()`.
   While `JsonWebToken::sign` uses `.expect()` with a message,
   both paths will panic at runtime if encoding fails, rather
   than returning an error through the service layer.

8. **`parse_token` panics on malformed headers.**
   `parse_token` calls `.expect()` on `to_str()` and blindly
   indexes into `split[1]`. A malformed `Authorization` header
   will panic the request handler.

9. **No token revocation mechanism.**
   There is no blacklist, version counter, or server-side session
   store. Once a JWT is signed, it is valid until `exp`. Password
   changes, role changes, and bans do not invalidate outstanding
   tokens.

10. **Scattered `jsonwebtoken` usage — no single module.**
    The `jsonwebtoken` crate is imported directly in three files
    (`services/authentication.rs`, `mailer.rs`, `services/user.rs`).
    There is no centralised JWT module that owns signing, verification,
    key management, and algorithm configuration. Changing the algorithm
    or key format requires touching multiple files.

11. **`BearerToken` extractor returns `Ok(None)` on missing header.**
    The Axum extractor never rejects a request for a missing
    `Authorization` header — it returns `Ok(Extract(None))` and
    defers the check downstream. This means every handler that
    requires authentication must remember to check for `None`
    and return `TokenNotFound` itself.

12. **`ClaimTokenPepper` naming is misleading.**
    In cryptography a "pepper" is a secret added to a password
    hash. Here it is used as an HMAC signing key, which is a
    fundamentally different concept. The name
    `user_claim_token_pepper` / `ClaimTokenPepper` obscures the
    actual role of the value.

## Options

### Option A — Incremental Cleanup (minimal scope)

Fix the most acute issues without changing the token format or
breaking API compatibility.

**Changes:**

- Extract a `jwt` module (`src/jwt.rs` or `src/jwt/mod.rs`) that
  centralises all `jsonwebtoken` usage: key loading, `sign`,
  `verify`, algorithm config.
- Move `VerifyClaims` into the new module alongside `UserClaims`.
- Make expiration durations configurable in `Auth` config
  (`session_token_lifetime_seconds`,
  `email_verification_token_lifetime_seconds`).
- Remove the redundant manual `exp` check — rely on the library's
  built-in validation.
- Replace `.unwrap()` / `.expect()` with `Result` propagation.
- Fix `parse_token` to return `Result` instead of panicking.
- Rename `ClaimTokenPepper` → `JwtSigningSecret` (or similar).
- Add `iss` and `sub` claims to `UserClaims`.

**Pros:**
- Small diff, low risk, no breaking API change.
- All existing tokens remain valid (backward-compatible).

**Cons:**
- Does not address the stale-role-in-token problem (#4).
- Does not address single-secret-for-all-purposes (#1).
- Does not add revocation (#9).
- HMAC-HS256 with a low-entropy secret remains (#2).

---

### Option B — Proper Claim Design + Per-Purpose Keys

Redesign the JWT claims to follow RFC 7519 best practices and
introduce separate signing keys per token purpose.

**Changes (includes all of Option A, plus):**

- Split the config secret into two independent keys:
  `auth.session_signing_key` and `auth.email_verification_signing_key`.
- Redesign `UserClaims` to standard registered claims:
  ```rust
  struct SessionClaims {
      sub: UserId,      // subject = user ID
      iss: String,      // "torrust-index"
      aud: String,      // "session"
      iat: u64,
      exp: u64,
      role: Role,       // admin / user
      username: String, // convenience, non-authoritative
  }
  ```
- Redesign `VerifyClaims` similarly with `aud: "email-verification"`.
- Re-validate the `role` / `administrator` flag from the database
  on every authenticated request (or cache with a short TTL) so
  the token role is advisory only.
- Enforce a minimum secret length at config validation time.

**Pros:**
- Purpose-separated keys: compromising one does not affect the other.
- Stale roles no longer grant elevated privileges.
- Standards-compliant claims improve interoperability.

**Cons:**
- **Breaking change** — existing session tokens become invalid on
  deploy (users must re-login).
- Config migration required (new key names).
- Re-checking the user role on every request adds a database
  round-trip (can be mitigated with a short-lived cache).

---

### Option C — Asymmetric Signing (RS256 / EdDSA)

Move from symmetric HMAC to an asymmetric algorithm.

**Changes (includes all of Option B, plus):**

- Replace `HS256` with `RS256` or `EdDSA`.
- Store a private key (PEM / PKCS#8) for signing and a
  corresponding public key for verification.
- Config provides a `auth.private_key_path` and
  `auth.public_key_path` (or inline PEM via env var).
- Only the signing service needs the private key; the verification
  layer (and potentially external services) only need the public
  key.
- Supports future use-cases like external microservices verifying
  tokens without sharing a secret, or JWKS endpoint publishing.

**Pros:**
- Strongest security posture — no shared secret.
- Enables zero-trust verification by third-party services.
- Aligns with modern OAuth 2.0 / OIDC practices.
- Supports key rotation via JWKS-style `kid` header.

**Cons:**
- Significantly higher complexity: key generation, storage, rotation.
- Larger tokens (RSA signatures are ~256 bytes vs. 32 for HMAC).
  EdDSA mitigates this (~64 bytes).
- Operational burden: deployers must generate and manage key pairs.
- Breaking change — same token-invalidation concern as Option B.
- `simple_asn1` / `time` pin issues (see ADR-T-005) may constrain
  which `jsonwebtoken` features can be enabled.

---

### Option D — Move to Opaque Session Tokens + Server-Side Store

Replace JWTs entirely with opaque session tokens backed by a
server-side session store.

**Changes:**

- Generate a cryptographically random opaque token on login
  (e.g., 256-bit via `rand`).
- Store a session record (token hash, user ID, expiry, role) in a
  new `torrust_sessions` database table (or Redis / in-memory cache).
- On each request, look up the token hash in the store, reject if
  absent or expired.
- Email-verification tokens can remain as purpose-specific JWTs
  (short-lived, no session semantics) or also become opaque +
  stored.
- Remove the `jsonwebtoken` dependency entirely (or keep it only
  for email-verification links).

**Pros:**
- Instant revocation — delete the row, token is dead.
- No stale-role problem — role is always read from the store/DB.
- No secret-key management for session tokens.
- Eliminates all JWT-specific bugs (claim design, algorithm
  confusion, etc.).

**Cons:**
- Every authenticated request requires a store lookup (DB or cache).
- New infrastructure dependency if using Redis; new migration if
  using the DB.
- Loses the statelessness benefit of JWTs.
- Larger scope — session management, garbage collection of expired
  rows, etc.
- Breaking change for any client that currently introspects the
  JWT payload.

---

### Option E — Hybrid (JWT + Server-Side Revocation List)

Keep JWTs for their stateless benefits but add a lightweight
server-side mechanism for revocation.

**Changes (includes all of Option B, plus):**

- Add a `token_generation` (or `jwt_epoch`) integer column to the
  `torrust_users` table. Increment it on password change, role
  change, or ban.
- Include `gen: u64` (the user's `token_generation` at sign time)
  in the JWT claims.
- On verification, compare the token's `gen` to the current
  database value; reject if stale.
- Optionally: maintain a small in-memory
  `HashMap<UserId, token_generation>` cache with a TTL of a few
  seconds, so the DB is not hit on every request.

**Pros:**
- Retains stateless JWT benefits for the common (non-revoked) case.
- Revocation is near-instant (one DB update per user).
- Smaller scope than full session-store migration.
- Compatible with either symmetric or asymmetric signing.

**Cons:**
- Still requires a DB/cache lookup per request (though cacheable).
- More complex than pure JWT or pure server-side sessions.
- Breaking change alongside Option B's claim redesign.

## Decision

**Option C — Asymmetric Signing with RS256**, implemented as a
phased rollout that subsumes Options A and B.

### Why Option C

1. **Dependency already supports it.** `jsonwebtoken 10.3.0` with
   `rust_crypto` already enables `rsa`, `pem`, `sha2`, and
   `use_pem`. No new crates, no feature-flag changes, no pin
   concerns (the `simple_asn1`/`time` pin issues from ADR-T-003
   were resolved in ADR-T-005).

2. **Strongest security posture.** Asymmetric signing eliminates
   the shared-secret problem entirely. Only the signing service
   holds the private key; verification requires only the public
   key. This is a strict improvement over HS256 with a low-entropy
   pepper.

3. **Future-proof.** Enables external microservices (e.g., a
   tracker, a frontend SSR server) to verify tokens without
   sharing a secret. A JWKS endpoint or `kid` header can be added
   later for key rotation without protocol changes.

4. **Subsumes Options A and B.** The phased plan below delivers
   all of Option A's cleanup and Option B's claim redesign as
   prerequisite steps before switching the algorithm.

### Why RS256 (not EdDSA)

- RS256 (`RSASSA-PKCS1-v1_5 + SHA-256`) is the most widely
  supported JWT algorithm across languages, libraries, and cloud
  services. Every JWT implementation is required to support it
  (RFC 7518 §3.1).
- EdDSA (Ed25519) produces smaller signatures (~64 bytes vs.
  ~256 bytes for RS256), but the size difference is negligible
  for authentication tokens transmitted once per request in an
  HTTP header.
- RS256 key generation and management are well-understood
  operationally (`openssl genrsa`).
- If EdDSA is desired in the future, the centralised `jwt` module
  makes the algorithm a single-point change.

### Why not Options D or E

- **Option D (opaque tokens):** Adds a mandatory server-side store
  (database table or Redis) on every request path. The project
  does not currently need instant revocation badly enough to
  justify the infrastructure cost and loss of statelessness.
- **Option E (hybrid revocation):** The `token_generation` column
  approach was originally deferred but has since been implemented
  in Phase 4. See the Phase 4 section below.

### Implementation Phases

#### Phase 1 — Structural Cleanup (Option A scope) ✅ Implemented

- ✅ Extract a `src/jwt.rs` module that centralises all
  `jsonwebtoken` usage: key loading, `sign`, `verify`, algorithm
  configuration.
- ✅ Move `UserClaims` and `VerifyClaims` into the new module.
- ✅ Replace `.unwrap()` / `.expect()` with `Result` propagation.
- ✅ Fix `parse_token` to return `Result` instead of panicking.
- ✅ Remove the redundant manual `exp` check (the library's
  `Validation` already handles it).
- ✅ Rename `ClaimTokenPepper` → `JwtSigningSecret`
  throughout config and code.
- ✅ Make expiration durations configurable:
  `session_token_lifetime_secs`,
  `email_verification_token_lifetime_secs`.

#### Phase 2 — Claim Redesign + Per-Purpose Keys (Option B scope) ✅ Implemented

- ✅ Redesign `UserClaims` → `SessionClaims`:
  ```rust
  struct SessionClaims {
      sub: UserId,      // subject = user ID
      iss: String,      // "torrust-index"
      aud: String,      // "session"
      iat: u64,
      exp: u64,
      role: Role,       // admin | user (advisory only)
      username: String, // convenience, non-authoritative
  }
  ```
- ✅ Redesign `VerifyClaims` with `aud: "email-verification"`.
- ✅ Split config into two independent keys:
  `auth.session_signing_key` and
  `auth.email_verification_signing_key`.
- ✅ Re-validate the user's role from the database on every
  authenticated request (the authorization service already does
  this via `get_role`) so the token role is advisory only.
- ✅ Enforce a minimum secret length (32 bytes) at config
  validation time. *(With Phase 3's move to RS256, this is now
  enforced implicitly: `EncodingKey::from_rsa_pem` /
  `DecodingKey::from_rsa_pem` reject invalid PEM at startup.)*
- **Breaking change:** existing HS256 tokens are invalidated;
  users must re-login.

#### Phase 3 — RS256 Asymmetric Signing (Option C scope) ✅ Implemented

- ✅ Replace `HS256` with `RS256` (`Algorithm::RS256`).
- ✅ Config provides:
  - `auth.private_key_path` (PEM / PKCS#8) for signing.
  - `auth.public_key_path` for verification.
  - Alternatively, inline PEM via environment variable
    (`auth.private_key_pem`, `auth.public_key_pem`).
- ~~Development key pair shipped at `share/default/jwt/` with loud
  startup warning when the default dev keys are used.~~
  *(Implemented, then superseded by Phase 5 — ephemeral
  auto-generated keys replace the shipped dev keys.)*
- ✅ Use `EncodingKey::from_rsa_pem` / `DecodingKey::from_rsa_pem`.
- ✅ Only the signing service loads the private key; the
  verification path uses the public key.
- ✅ A `kid` (Key ID) is included in every JWT header (SHA-256
  fingerprint of the public key) to support future key rotation.
- **Breaking change:** existing HS256 tokens and config
  (`session_signing_key`, `email_verification_signing_key`) are
  no longer supported. Deployers must generate an RSA key pair
  and update their configuration.

#### Phase 4 — Optional Revocation (Option E scope) ✅ Implemented

- ✅ Add a `token_generation` column (default `0`) to
  `torrust_users`.
- ✅ Include `gen` in `SessionClaims`; reject tokens whose `gen`
  is older than the current database value.
- ✅ Increment `token_generation` on password change, role change
  (admin grant), and ban.
- ✅ Validation performed in the `Authentication` web layer
  (`get_user_id_from_bearer_token`), the `verify_token_handler`,
  and the `renew_token` service method.
  *Defence in depth:* the generation check is intentionally
  repeated at each entry point rather than consolidated into a
  single layer, so that no call path can accidentally bypass
  revocation.
- **Breaking change:** existing tokens without a `gen` claim will
  fail deserialization and be rejected (users re-login once).

#### Phase 5 — Ephemeral Auto-Generated Keys (default) ✅ Implemented

##### Behaviour

- Remove the shipped development key pair from `share/default/jwt/`.
- On startup, if no key paths or PEM values are configured,
  **auto-generate an RSA-2048 key pair in memory** via the `rsa`
  crate's `RsaPrivateKey::new(&mut OsRng, 2048)`.
- The generated keys are held only in process memory and are
  **never written to disk**. On shutdown (or crash) the keys are
  lost; all outstanding tokens become unverifiable and users must
  re-login.
- Log a clear informational message at startup:
  `"Using ephemeral auto-generated RSA key pair. Sessions will
  not survive server restarts. To persist sessions, configure
  auth.private_key_path / auth.public_key_path."`
- No development-mode keys exist in the repository. There is no
  distinction between "dev" and "prod" key material — only
  between ephemeral (default) and host-supplied (persistent).
- For **persistent sessions across restarts**, the deployer
  generates their own RSA key pair and configures the paths or
  environment variables as described in Phase 3.
- **No breaking change for existing Phase 3 deployers** who
  already supply their own key pair — their configuration
  continues to work as before. Only the *default* behaviour
  changes (from shipped dev keys to ephemeral keys).

##### New dependencies

The `rsa` crate (already a transitive dependency via
`jsonwebtoken`'s `rust_crypto` feature) must be added as a
**direct** dependency in `Cargo.toml` along with `rand` (for
`OsRng`). PEM export requires the `pkcs8` + `pem` features on
`rsa` (for `EncodePrivateKey::to_pkcs8_pem`) and `spki` (for
`EncodePublicKey::to_public_key_pem`).

##### Key generation details

`RsaPrivateKey::new()` is CPU-intensive (~100-300 ms for 2048
bits). Because `JsonWebToken::new()` is `async`, the generation
must be wrapped in `tokio::task::spawn_blocking` to avoid stalling
the async executor.

After generation, the private and public keys are exported to
in-memory PEM byte vectors via:
```rust
use rsa::pkcs8::EncodePrivateKey;
use rsa::pkcs8::LineEnding;
use spki::EncodePublicKey;

let private_pem = private_key
    .to_pkcs8_pem(LineEnding::LF)
    .expect("PEM export");
let public_pem = private_key
    .to_public_key()
    .to_public_key_pem(LineEnding::LF)
    .expect("PEM export");
```
These PEM bytes are then passed to `EncodingKey::from_rsa_pem` /
`DecodingKey::from_rsa_pem` exactly as the host-supplied path does
today.

##### Interface changes

- **`Auth::resolve_private_key_pem()` / `resolve_public_key_pem()`**
  currently **panic** when no key is found. These methods must
  change their return type to `Option<Vec<u8>>` so the caller
  (`JsonWebToken::new`) can distinguish "no key configured" from
  "key configured but invalid".
- **`Auth::default()`** must set `private_key_path` and
  `public_key_path` to `None` (not the former
  `./share/default/jwt/…` paths). The `DEFAULT_PRIVATE_KEY_PATH`
  and `DEFAULT_PUBLIC_KEY_PATH` constants are removed.
- **`JsonWebToken::new()`** gains a new branch: when both
  `resolve_*` methods return `None`, it generates an ephemeral
  key pair (via `spawn_blocking`) and logs the informational
  message.

##### Files affected by dev-key removal

Removing `share/default/jwt/` and the default-path constants
touches the following files (non-exhaustive):

| File | Change |
|---|---|
| `share/default/jwt/private.pem`, `public.pem` | Delete |
| `src/config/v2/auth.rs` | Remove `DEFAULT_*_KEY_PATH` constants; change defaults to `None`; return `Option` from `resolve_*` |
| `src/jwt.rs` | Add ephemeral-generation branch in `JsonWebToken::new()` |
| `src/lib.rs` | Update doc-comment example config (remove key paths from default) |
| `src/tests/jwt.rs` | `jwt_service()` helper uses ephemeral path (no path overrides) |
| `tests/fixtures/default_configuration.toml` | Remove `private_key_path` / `public_key_path` lines |
| `.env.local` | Remove `AUTH__PRIVATE_KEY_PATH` / `AUTH__PUBLIC_KEY_PATH` overrides |
| `share/default/config/index.development.sqlite3.toml` | Remove key-path lines |
| `contrib/dev-tools/container/e2e/sqlite/install.sh` | Remove `cp …/jwt/*.pem` line |
| `contrib/dev-tools/container/e2e/mysql/install.sh` | Remove `cp …/jwt/*.pem` line |
| `compose.yaml` | Remove or comment out `AUTH__PRIVATE_KEY_PATH` / `AUTH__PUBLIC_KEY_PATH` env vars |
| `src/web/api/server/v1/contexts/user/mod.rs` | Update module-level doc example |

#### Phase 6 — `generate-auth-keypair` CLI + Container Auto-Generation

##### Motivation

Phases 3 and 5 require deployers who want **persistent sessions**
to generate an RSA key pair externally (e.g., via `openssl`).
This creates an operational dependency on a tool that may not be
present in minimal container images (the runtime image is
distroless). Rather than adding `openssl` to the container, the
key generation capability is built into the project itself — the
`rsa` crate is already a direct dependency.

The goal is zero-friction persistent sessions in the container:
on first boot, if no keys exist on the `/etc/torrust/index`
volume, the entry script generates them automatically. Subsequent
restarts reuse the same keys, so sessions survive. Hosts who want
their own keys either pre-populate the volume before the first
start, or overwrite the generated keys and restart.

##### CLI binary — `torrust-generate-auth-keypair`

A new binary `torrust-generate-auth-keypair`
(`src/bin/generate_auth_keypair.rs`) generates an RSA-2048 key
pair and writes both PEM blocks to **stdout**. Design constraints:

- **Stdout must be piped.** The tool refuses to run if stdout is
  a terminal (`std::io::stdout().is_terminal()`), printing a
  usage hint to stderr and exiting with code 1. This prevents
  accidental display of key material on screen.
- **Private key on stdout first, then the public key**, each in
  standard PEM (Base64-encoded PKCS#8 / SPKI) format. The two
  blocks are self-delimiting via their `-----BEGIN …-----` /
  `-----END …-----` markers.
- **Diagnostic message on stderr** confirming the key was
  generated (type, bit size).
- Uses `clap` (already a dependency) for `--help` and future
  extensibility (e.g., `--bits`, `--out-dir`).
- Reuses the same `rsa` + `pkcs8` code path as the ephemeral
  generator in `src/jwt.rs`.

##### Container integration

The container entry script (`share/container/entry_script_sh`)
auto-generates persistent keys on first boot:

```sh
# Generate auth keys if not already present on the volume.
private_key="/etc/torrust/index/private.pem"
public_key="/etc/torrust/index/public.pem"

if [ ! -f "$private_key" ] || [ ! -f "$public_key" ]; then
    torrust-generate-auth-keypair > /tmp/auth_keys.pem 2>/dev/null
    sed -n '/BEGIN PRIVATE/,/END PRIVATE/p' /tmp/auth_keys.pem > "$private_key"
    sed -n '/BEGIN PUBLIC/,/END PUBLIC/p'   /tmp/auth_keys.pem > "$public_key"
    rm -f /tmp/auth_keys.pem
    chown torrust:torrust "$private_key" "$public_key"
    chmod 0400 "$private_key"
    chmod 0440 "$public_key"
fi
```

Because `/etc/torrust/index` is a declared `VOLUME`, the
generated keys persist across container restarts and image
upgrades. Sessions survive as long as the volume is retained.

All container configuration files (`share/default/config/`) set:
```toml
[auth]
private_key_path = "/etc/torrust/index/private.pem"
public_key_path  = "/etc/torrust/index/public.pem"
```

##### Containerfile changes

The `torrust-generate-auth-keypair` binary is copied into `/usr/bin/` in
both the debug and release runtime images alongside
`torrust-index` and `health_check`:

```dockerfile
# Extract and Test (debug)
RUN mkdir -p /app/bin/; \
  cp -l /test/src/target/debug/torrust-index /app/bin/torrust-index; \
  cp -l /test/src/target/debug/torrust-generate-auth-keypair /app/bin/torrust-generate-auth-keypair

# Extract and Test (release)
RUN mkdir -p /app/bin/; \
  cp -l /test/src/target/release/torrust-index /app/bin/torrust-index; \
  cp -l /test/src/target/release/health_check /app/bin/health_check; \
  cp -l /test/src/target/release/torrust-generate-auth-keypair /app/bin/torrust-generate-auth-keypair
```

##### Host-supplied keys (custom key workflow)

Hosts who want to use their own RSA key pair have two options:

1. **Pre-supply before first boot.** Mount or copy key files into
   the `/etc/torrust/index` volume before starting the container.
   The entry script's existence check (`[ ! -f … ]`) will skip
   generation and the server will use the host's keys directly.

2. **Overwrite after first boot.** Let the container auto-generate
   keys on first boot since it "just works". Later, replace the
   generated PEM files on the volume with the host's own keys and
   restart the container. The server picks up the new keys; any
   tokens signed with the old keys are invalidated (users
   re-login once).

##### Usage outside containers

```sh
# Generate and split into two files:
cargo run --bin torrust-generate-auth-keypair \
  | tee >(sed -n '/BEGIN PRIVATE/,/END PRIVATE/p' > private.pem) \
        >(sed -n '/BEGIN PUBLIC/,/END PUBLIC/p'   > public.pem) \
        > /dev/null
```

##### Files affected

| File | Change |
|---|---|
| `src/bin/torrust-generate-auth-keypair.rs` | New binary |
| `Cargo.toml` | No change — auto-discovered by Cargo |
| `Containerfile` | Copy `torrust-generate-auth-keypair` into `/app/bin/` in both debug and release stages |
| `share/container/entry_script_sh` | Add key-generation block before `exec su-exec` |
| `share/default/config/index.container.sqlite3.toml` | Add `private_key_path` / `public_key_path` to `[auth]` |
| `share/default/config/index.container.mysql.toml` | Add `private_key_path` / `public_key_path` to `[auth]` |
| `share/default/config/index.public.e2e.container.sqlite3.toml` | Add `private_key_path` / `public_key_path` to `[auth]` |
| `share/default/config/index.public.e2e.container.mysql.toml` | Add `private_key_path` / `public_key_path` to `[auth]` |
| `share/default/config/index.private.e2e.container.sqlite3.toml` | Add `private_key_path` / `public_key_path` to `[auth]` |

##### No breaking changes

This phase adds a new binary and updates the container entry
script. Bare-metal deployments without key paths configured
continue to use Phase 5's ephemeral in-memory keys. Container
deployments gain automatic persistent keys with no manual setup.

### Configuration Migration

Deployers upgrading across Phase 2 / Phase 3 must:

1. Generate an RSA key pair — either via
   `cargo run --bin torrust-generate-auth-keypair | …` (see Phase 6) or
   externally (`openssl genrsa -out private.pem 2048` and
   `openssl rsa -in private.pem -pubout -out public.pem`).
2. Update the config to reference the key paths (or set env vars).
3. Accept that existing sessions will be invalidated (users
   re-login once).

A migration guide will accompany the release that ships Phase 3.

With Phase 5, steps 1–2 become **optional** for bare-metal
deployments. Without explicit key configuration the server
auto-generates ephemeral keys and functions immediately —
sessions simply do not survive restarts.

With Phase 6, **container deployments handle key generation
automatically.** The entry script generates keys to the
`/etc/torrust/index` volume on first boot; the container configs
already point to the generated paths. No manual key generation or
config editing is required. Sessions persist across restarts as
long as the volume is retained.

Note: the **serialized default config** changes in Phase 5 — the
bare-metal `[auth]` section will no longer contain
`private_key_path` / `public_key_path` entries. Container configs
*do* include these paths (pointing to `/etc/torrust/index/`).
Deployers who generate their config from defaults should be aware
of this difference. Existing configs that explicitly set these
fields are unaffected.

## Consequences

- Existing user sessions **will be invalidated** when Phase 2
  ships (claim format change) and again if key material changes
  in Phase 3. Users must re-login.
- **Container deployments** auto-generate persistent keys on
  first boot (Phase 6). Sessions survive restarts with no manual
  setup. Hosts who want their own keys pre-populate the volume or
  overwrite the generated keys and restart.
- **Bare-metal deployments** without key paths configured use
  ephemeral in-memory keys (Phase 5) — sessions do not survive
  restarts. Deployers who want persistent sessions generate a key
  pair via `torrust-generate-auth-keypair` (Phase 6) or `openssl` and
  configure the paths.
- Token revocation via a `token_generation` counter is included
  (Phase 4 / Option E). Password changes, role changes, and bans
  increment the counter and invalidate outstanding tokens.
- The centralised `jwt` module makes future algorithm changes
  (e.g., migrating to EdDSA) a localised, single-module change.
- External services can verify tokens using only the public key,
  enabling zero-trust verification without secret sharing.

## Testing Strategy

The repository **does not ship any pre-generated RSA key material**.
Tests exercise three key-provisioning modes:

### Crate-level tests (`src/tests/jwt.rs`)

The existing `jwt_service()` helper constructs a `JsonWebToken`
with **no key paths configured**, exercising the ephemeral
in-memory generation code path (Phase 5). All round-trip, claim,
and error-path tests work unchanged — they only need a valid
`JsonWebToken` instance, regardless of how the keys were
provisioned.

### Isolated e2e tests (bare-metal path)

Isolated e2e tests (the default `cargo test` mode) start an
in-process server with a `TempDir`-based ephemeral configuration.
No key paths are configured, so the server auto-generates keys in
memory. Authentication works for the lifetime of the test process.
No special setup is required.

### Container e2e tests (persistent-key path)

Container e2e tests (`compose.yaml`) exercise the production-like
flow where `torrust-generate-auth-keypair` runs in the entry script:

1. The entry script detects no keys on the `/etc/torrust/index`
   volume and runs `torrust-generate-auth-keypair` to create them.
2. The container configs point `auth.private_key_path` and
   `auth.public_key_path` at the generated files.
3. The server starts with host-supplied (volume-persisted) keys.
4. E2e tests run the full auth round-trip: register, login,
   authenticated requests.

Because the keys live on a volume, restarting the container
reuses the same key pair — proving session persistence across
restarts (the production contract).

### Host-supplied-key e2e test

A dedicated e2e test verifies that externally generated keys are
accepted. The test has an external dependency on **`openssl`**
(must be on `$PATH`).

**Test outline (`tests/e2e/web/api/v1/contexts/user/`)**:

1. **Generate a fresh RSA key pair via `openssl`** into a
   temporary directory (`tempfile::TempDir`):
   ```sh
   openssl genrsa -out "$tmpdir/private.pem" 2048
   openssl rsa -in "$tmpdir/private.pem" -pubout -out "$tmpdir/public.pem"
   ```
   Executed with `std::process::Command`. The test is
   `#[ignore]`-gated (or behind a feature flag / env var) so CI
   runners without `openssl` can skip it gracefully.

2. **Start a test environment** with config overrides pointing
   `auth.private_key_path` and `auth.public_key_path` at the
   generated files.

3. **Perform a full auth round-trip:**
   - Register a user.
   - Log in and receive a session JWT.
   - Call an authenticated endpoint using the token.
   - Verify the response succeeds (the host-supplied key pair is
     used for signing and verification).

4. **Restart the server** (same key pair, same temp dir) and
   confirm the previously issued JWT is **still valid** — proving
   session persistence across restarts.

5. **Cleanup** — the `TempDir` drops automatically, removing the
   generated keys.

This test proves:
- The repository contains no key material and the server boots
  without shipped keys.
- `openssl`-generated keys are accepted by
  `EncodingKey::from_rsa_pem` / `DecodingKey::from_rsa_pem`.
- Sessions persist across restarts when the deployer supplies
  their own keys.

## Remaining Issues

- **Problem #11 (`BearerToken` extractor returns `Ok(None)`).**
  ✅ **Resolved.** The `BearerToken` extractor now implements
  `FromRequestParts` directly and **rejects** missing
  (`AuthError::TokenNotFound`) or malformed
  (`AuthError::TokenInvalid`) `Authorization` headers at the
  extraction boundary. The `Extract` wrapper has been removed.
  `ExtractLoggedInUser` uses `BearerToken` directly (fails if
  missing). `ExtractOptionalLoggedInUser` catches the rejection
  and returns `None` for anonymous requests.
  `Authentication::get_user_id_from_bearer_token` now takes
  `BearerToken` (not `Option<BearerToken>`), eliminating the
  `None`-handling indirection.
