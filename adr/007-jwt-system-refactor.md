# ADR-T-007: Refactor the JWT System

**Status:** Phases 1–7 implemented
**Date:** 2026-04-14
**Updated:** 2026-04-16

## Context

JWT handling was spread across four locations with two distinct
claim types, sharing a single HMAC-HS256 signing secret:

| Location | Purpose | Claim type |
|---|---|---|
| `services::authentication::JsonWebToken` | Sign/verify session tokens | `UserClaims` |
| `web::api::server::v1::auth::Authentication` | Wrapper delegating to `JsonWebToken` | `UserClaims` |
| `mailer::Service::get_verification_url` | Sign email-verification tokens | `VerifyClaims` |
| `services::user::RegistrationService::verify_email` | Verify email-verification tokens | `VerifyClaims` |

### Problems

1. **Single shared secret.** Session and email-verification JWTs
   shared one HMAC key (`user_claim_token_pepper`). Compromising
   one compromised both.

2. **Low-entropy HMAC key.** A human-readable string (default:
   `"MaxVerstappenWC2021"`) used directly as
   `EncodingKey::from_secret`. No minimum entropy, no key
   derivation, no asymmetric support.

3. **Missing registered claims.** `UserClaims` had only
   `{ user, exp }` — no `iss`, `aud`, `sub`, or `iat`.
   `VerifyClaims` had `iss` and `sub` but not `aud`.

4. **Stale privileges in tokens.** The `administrator` flag was
   trusted from the token. Role changes were invisible until
   token expiry (two weeks).

5. **Hard-coded expiration.** Session: two weeks.
   Email verification: ~10 years. Renewal threshold: one week.
   None configurable.

6. **Redundant expiration check.** Library validation and a
   manual `exp` check could disagree on clock skew.

7. **Panics on encode failure.** `.unwrap()` / `.expect()` on
   `encode` in both signing paths.

8. **Panics on malformed headers.** `.expect()` and blind
   indexing in `parse_token`.

9. **No revocation.** Once signed, tokens were valid until `exp`.

10. **Scattered `jsonwebtoken` usage.** Imported in three files
    with no centralised module.

11. **Extractor swallowed missing headers.** `BearerToken`
    returned `Ok(None)` on a missing `Authorization` header,
    pushing the auth check into every handler.

12. **Misleading naming.** `ClaimTokenPepper` used cryptographic
    pepper terminology for what was an HMAC signing key.

## Options Considered

### A — Incremental Cleanup

Centralise `jsonwebtoken`, fix panics, make durations
configurable, rename types. Small diff, no breaking change.

Does not address stale roles (#4), single secret (#1),
revocation (#9), or low-entropy keys (#2).

### B — Claim Redesign + Per-Purpose Keys

Option A plus: per-purpose signing keys, RFC 7519 claims,
database-authoritative roles. Breaking change (re-login
required).

### C — Asymmetric Signing (RS256)

Option B plus: replace HS256 with RS256. Private key signs;
public key verifies. Strongest posture, highest complexity.

### D — Opaque Tokens + Server-Side Store

Replace JWTs entirely with random opaque tokens. Instant
revocation, no key management. Loses statelessness; requires a
store lookup on every request.

### E — Hybrid (JWT + Generation Counter)

Keep JWTs; add a per-user `token_generation` counter. Increment
on password/role/ban changes. Near-instant revocation with
minimal infrastructure.

## Decision

**Option C (RS256)** as a phased rollout that subsumes A, B,
and E.

### Rationale

- **Already supported.** `jsonwebtoken 10.3.0` with
  `rust_crypto` enables `rsa`, `pem`, `sha2` — no new crates.
- **Strongest posture.** Asymmetric signing eliminates the
  shared-secret problem; only the signing service holds the
  private key.
- **Future-proof.** External services verify tokens with the
  public key alone. JWKS / `kid` rotation can be added later.
- **RS256 over EdDSA.** Most widely supported JWT algorithm
  (RFC 7518 §3.1); operationally well-understood key management.
  EdDSA's smaller signatures are negligible for auth tokens.
- **Not Option D.** Opaque tokens require a mandatory store
  lookup and session management infrastructure the project
  doesn't need.

## Implementation

### Phase 1 — Structural Cleanup ✅

Extracted `src/jwt.rs` centralising all `jsonwebtoken` usage.
Moved claim types into the new module. Replaced panicking
`.unwrap()` / `.expect()` with `Result` propagation. Fixed
`parse_token` to return `Result`. Removed the redundant manual
`exp` check. Renamed `ClaimTokenPepper` → `JwtSigningSecret`.
Made expiration durations configurable:
`session_token_lifetime_secs`,
`email_verification_token_lifetime_secs`.

### Phase 2 — Claim Redesign + Per-Purpose Keys ✅

Redesigned `UserClaims` → `SessionClaims` with RFC 7519
registered claims (`sub`, `iss`, `aud`, `iat`, `exp`) plus
advisory `role` and `username` fields:

```rust
struct SessionClaims {
    sub: UserId,      // subject = user ID
    iss: String,      // "torrust-index"
    aud: String,      // "session"
    iat: u64,
    exp: u64,
    role: String,     // advisory — non-authoritative
    username: String, // advisory — non-authoritative
    gen: u64,         // token generation (added in Phase 4)
}
```

Redesigned `VerifyClaims` with `aud: "email-verification"`.
Split config into two independent signing keys. Role is
re-validated from the database on every authenticated request
(advisory only in token).

**Breaking:** existing HS256 tokens invalidated.

### Phase 3 — RS256 Asymmetric Signing ✅

Replaced HS256 with RS256. Config provides
`auth.private_key_path` / `auth.public_key_path` (or inline
PEM via env vars). Uses `EncodingKey::from_rsa_pem` /
`DecodingKey::from_rsa_pem`. Only the signing service loads the
private key. A `kid` (SHA-256 fingerprint of the public key)
is included in every JWT header for future key rotation.

**Breaking:** HS256 config keys no longer supported.

### Phase 4 — Token Revocation ✅

Added a `token_generation` column (default `0`) to
`torrust_users`. `SessionClaims` includes a `gen` field; tokens
whose `gen` does not **exactly** match the database value are
rejected (`!=`, not `<` — tokens are also rejected if the
database generation *decreases*, e.g. restore from backup).

State-changing operations use atomic database methods so the
state change and generation bump either both apply or neither
does:

| Operation | Method | Mechanism |
|---|---|---|
| Password change | `change_user_password_and_revoke_tokens` | Transaction |
| Admin grant | `grant_admin_role_and_revoke_tokens` | Single UPDATE |
| Ban | `ban_user_and_revoke_tokens` | Transaction |

The ban table is also checked as a secondary guard: if
`token_generation` somehow matches despite an active ban,
`is_user_banned` catches it.

Validation is consolidated into `JsonWebToken::validate_session`
(Phase 7).

**Breaking:** tokens without a `gen` claim fail deserialization.

### Phase 5 — Ephemeral Auto-Generated Keys ✅

When no key paths or PEM values are configured, an RSA-2048 key
pair is auto-generated in memory at startup via
`RsaPrivateKey::new(&mut OsRng, 2048)` (wrapped in
`spawn_blocking`). Keys are never written to disk; sessions do
not survive restarts. For persistent sessions, the deployer
supplies their own key pair.

Removed shipped development keys. `Auth::default()` sets key
paths to `None`. The `resolve_*` methods return
`Option<Vec<u8>>` so callers distinguish "no key configured"
from "invalid key."

#### Dependencies

The `rsa` crate (already transitive via `jsonwebtoken`) is a
direct dependency along with `rand`. PEM export uses
`EncodePrivateKey::to_pkcs8_pem` and
`EncodePublicKey::to_public_key_pem` from transitive `pkcs8`
and `spki`.

### Phase 6 — `auth-keypair` CLI ✅

A binary `torrust-index-auth-keypair` (initially shipped as
`torrust-generate-auth-keypair`) generates an RSA-2048 key
pair to stdout. As of ADR-T-009 Phase 2 the binary lives in
its own workspace crate at
[`packages/index-auth-keypair/`](../packages/index-auth-keypair/);
the earlier `src/bin/generate_auth_keypair.rs` location no
longer exists. Design:

- Refuses to run if stdout is a terminal (exit code 2).
- Emits a single JSON object
  `{"private_key_pem": "...", "public_key_pem": "..."}`
  on stdout ([ADR-T-010](010-global-command-line-output-contract.md)). The original raw-PEM
  output was replaced in Phase 2.
- Diagnostics on stderr via `tracing` (NDJSON); `--debug` for verbose.
- Uses `clap` for CLI.

#### Container integration

The entry script (`share/container/entry_script_sh`)
auto-generates persistent keys on first boot into
`/etc/torrust/index/auth/`. Hardening:

- `mkdir -p` with `0700` before any key material is written.
- `mktemp` + `chmod 0600` for the intermediate file.
- `[ ! -s … ]` (existence + non-empty) guards against
  zero-byte files from interrupted prior runs.
- `trap … EXIT` ensures temp file cleanup.
- `jq -r .private_key_pem` / `jq -r .public_key_pem` extract
  the PEM blocks from the helper's JSON output (post
  ADR-T-009 Phase 2; the original implementation used `sed`
  against raw PEM markers).
- Errors on stderr (visible in `docker logs`); non-zero exit
  on failure.
- **TOCTOU note:** if two containers race against the same
  volume, both could pass the check and overwrite each other's
  keys. Mitigate with `flock` if needed; single-container
  deployments are the norm.

Container configs set `auth.private_key_path` /
`auth.public_key_path` to the generated paths. Sessions persist
via the `/etc/torrust/index` volume.

#### Containerfile

`torrust-index-auth-keypair` is copied into `/usr/bin/` in
both the debug and release runtime images alongside
`torrust-index` and (release only) `torrust-index-health-check`.
The debug image deliberately omits the health-check binary —
see [`docs/containers.md`](../docs/containers.md#debug-image-healthcheck).

#### Host-supplied keys

Two workflows:

1. **Pre-supply:** mount or copy keys to the volume before first
   boot. The `[ ! -s … ]` check skips generation.

2. **Overwrite:** let the container auto-generate on first boot,
   then replace the PEM files and restart.

#### Usage outside containers

```sh
tmpfile=$(mktemp /tmp/auth_keys.XXXXXX)
chmod 0600 "$tmpfile"
cargo run -p torrust-index-auth-keypair > "$tmpfile"
jq -r .private_key_pem "$tmpfile" > private.pem
jq -r .public_key_pem  "$tmpfile" > public.pem
rm -f "$tmpfile"
```

The helper emits a single JSON object on stdout, so any
JSON-aware consumer (`jq`, `python -m json.tool`, a
`serde_json::from_reader::<KeypairOutput>` in Rust) works.
The earlier `sed` PEM-marker recipe is no longer applicable
because newlines inside the PEM bodies are JSON-escaped.

### Phase 7 — Consolidate Session Validation ✅

#### Problem

Phase 4's validation logic — verify JWT, check generation, check
ban — is copy-pasted at three call sites:

| Entry point | Location |
|---|---|
| `Authentication::get_user_id_from_bearer_token` | `web::api::server::v1::auth` |
| `verify_token_handler` | `web::api::server::v1::contexts::user::handlers` |
| `Service::renew_token` | `services::authentication` |

Each site independently re-implements the same sequence: verify
JWT → fetch `token_generation` → compare `gen` → check ban
table. This was originally framed as "defence in depth," but all
three sites are at the same architectural layer performing
identical checks. The duplication is a maintenance hazard, not a
safety net — a logic fix must be applied in three places (this
already happened with the `<` → `!=` correction in Phase 4), and
a new entry point can omit the checks entirely.

#### Design: correct by construction

Replace duplication with a single validation function that is the
**only way** to obtain validated `SessionClaims`. A new entry
point cannot forget the checks because the type system forces it
through the single code path.

All three structs that need session validation (`Authentication`,
`Service`, and handlers via `AppData`) share the same
`Arc<JsonWebToken>` and `Arc<Box<dyn Database>>`. The
consolidated function lives on `JsonWebToken` itself — the
centralised JWT module from Phase 1:

```rust
impl JsonWebToken {
    /// Verify a session JWT and validate it against the database.
    ///
    /// This is the **sole entry point** for session-token
    /// validation. It verifies the JWT signature and expiry,
    /// checks the token generation counter, and rejects banned
    /// users.
    pub async fn validate_session(
        &self,
        db: &dyn Database,
        token: &str,
    ) -> Result<SessionClaims, AuthError> {
        let claims = self.verify(token)?;

        let current_gen = db
            .get_token_generation(claims.sub)
            .await?;

        if claims.token_gen != current_gen {
            return Err(AuthError::TokenRevoked);
        }

        if db.is_user_banned(claims.sub).await.unwrap_or(false) {
            return Err(AuthError::TokenRevoked);
        }

        Ok(claims)
    }
}
```

#### Callers after consolidation

**`Authentication::get_user_id_from_bearer_token`** — delegates
directly; remove `validate_token_generation`:

```rust
pub async fn get_user_id_from_bearer_token(
    &self,
    token: BearerToken,
) -> Result<UserId, AuthError> {
    let claims = self.json_web_token
        .validate_session(&*self.database, token.as_str())
        .await?;
    Ok(claims.sub)
}
```

**`verify_token_handler`** — replaces inline verify + check
sequence:

```rust
pub async fn verify_token_handler(
    State(app_data): State<Arc<AppData>>,
    extract::Json(token): extract::Json<JsonWebToken>,
) -> Response {
    match app_data.json_web_token
        .validate_session(&*app_data.database, &token.token)
        .await
    {
        Ok(_) => axum::Json(OkResponseData {
            data: "Token is valid.".to_string(),
        })
        .into_response(),
        Err(error) => error.into_response(),
    }
}
```

**`Service::renew_token`** — replaces inline verify + check
sequence:

```rust
pub async fn renew_token(
    &self,
    token: &str,
) -> Result<(String, UserCompact), AuthError> {
    const ONE_WEEK_IN_SECONDS: u64 = 604_800;

    let claims = self.json_web_token
        .validate_session(&*self.database, token)
        .await?;

    let user_compact = self.user_repository
        .get_compact(&claims.sub)
        .await
        .map_err(|err| match err {
            Error::UserNotFound => AuthError::UserNotFound,
            err => AuthError::from(err),
        })?;

    let token = match claims.exp - clock::now() {
        x if x < ONE_WEEK_IN_SECONDS => {
            self.json_web_token
                .sign(user_compact.clone(), claims.token_gen)
                .await?
        }
        _ => token.to_string(),
    };

    Ok((token, user_compact))
}
```

#### Files affected

| File | Change |
|---|---|
| `src/jwt.rs` | Add `validate_session` method on `JsonWebToken` |
| `src/web/api/server/v1/auth.rs` | `get_user_id_from_bearer_token` delegates; remove `validate_token_generation` |
| `src/web/api/server/v1/contexts/user/handlers.rs` | `verify_token_handler` delegates |
| `src/services/authentication.rs` | `renew_token` delegates |

**No breaking change** — internal refactor only.

## Configuration Migration

Deployers upgrading across Phases 2–3 must:

1. Generate an RSA key pair — via
   `torrust-index-auth-keypair` (Phase 6) or `openssl`.
2. Update config to reference key paths (or set env vars).
3. Accept session invalidation (users re-login once).

With Phase 5, steps 1–2 are **optional** for bare-metal
deployments — ephemeral keys are auto-generated; sessions just
don't survive restarts.

With Phase 6, **container deployments handle key generation
automatically** on first boot. No manual setup required.

Note: the serialized default config for bare-metal deployments
no longer contains `private_key_path` / `public_key_path`
entries. Container configs *do* include these paths (pointing to
`/etc/torrust/index/auth/`). Existing configs that explicitly
set these fields are unaffected.

## Consequences

- Existing sessions **invalidated** at Phase 2 (claim format)
  and Phase 3 (algorithm change). Users re-login.
- **Container:** auto-generated persistent keys on first boot
  (Phase 6). Sessions survive restarts.
- **Bare-metal (no config):** ephemeral in-memory keys
  (Phase 5). Sessions do not survive restarts.
- **Bare-metal (with keys):** persistent sessions via
  deployer-supplied key pair.
- Token revocation via `token_generation` counter (Phase 4).
  Password changes, role changes, and bans invalidate
  outstanding tokens.
- Centralised `jwt` module makes future algorithm changes
  (e.g. EdDSA) a single-module edit.
- External services verify tokens using only the public key.
- The `BearerToken` extractor now rejects missing or malformed
  `Authorization` headers at extraction time (Problem #11).
  `ExtractOptionalLoggedInUser` catches the rejection for
  anonymous endpoints.
- Session validation consolidated into a single code path
  (Phase 7). New authentication entry points cannot bypass
  revocation or ban checks.

## Testing Strategy

The repository ships **no pre-generated RSA key material**.
Tests exercise three provisioning modes:

### Crate-level (`src/tests/jwt.rs`)

The `jwt_service()` helper constructs `JsonWebToken` with no
key paths, exercising ephemeral in-memory generation (Phase 5).

### Isolated e2e (bare-metal)

Tests start an in-process server with a `TempDir`
configuration. No key paths → auto-generated keys.
Authentication works for the test lifetime.

### Container e2e (persistent keys)

`compose.yaml` tests exercise the production flow: entry script
generates keys to the volume, server starts with file-supplied
keys, e2e auth round-trip runs.

### Host-supplied-key e2e

A `#[ignore]`-gated test verifies externally-generated keys
(requires `openssl` on `$PATH`): generates a key pair into a
temp dir, starts a server with those keys, runs a full auth
round-trip, restarts the server, and confirms previously-issued
tokens are still valid.
