# ADR-T-007: Refactor the JWT System

**Status:** Phase 2 implemented
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
  approach is elegant but adds complexity that can be layered on
  later without changing the token format. It remains a valid
  follow-up if revocation becomes a priority.

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
  validation time.
- **Breaking change:** existing HS256 tokens are invalidated;
  users must re-login.

#### Phase 3 — RS256 Asymmetric Signing (Option C scope)

- Replace `HS256` with `RS256` (`Algorithm::RS256`).
- Config provides:
  - `auth.private_key_path` (PEM / PKCS#8) for signing.
  - `auth.public_key_path` for verification.
  - Alternatively, inline PEM via environment variable.
- Generate a default development key pair on first run (with a
  loud warning) so the zero-config experience is preserved for
  local development.
- Use `EncodingKey::from_rsa_pem` / `DecodingKey::from_rsa_pem`.
- Only the signing service loads the private key; the
  verification path uses the public key.
- Add a `kid` (Key ID) field to the JWT header to support future
  key rotation.

#### Future — Optional Revocation (Option E scope)

- Add a `token_generation` column to `torrust_users`.
- Include `gen` in `SessionClaims`; reject stale generations on
  verify.
- Increment generation on password change, role change, or ban.
- This phase is independent and can be shipped whenever revocation
  becomes a priority.

### Configuration Migration

Deployers upgrading across Phase 2 / Phase 3 must:

1. Generate an RSA key pair (e.g.,
   `openssl genrsa -out private.pem 2048` and
   `openssl rsa -in private.pem -pubout -out public.pem`).
2. Update the config to reference the key paths (or set env vars).
3. Accept that existing sessions will be invalidated (users
   re-login once).

A migration guide will accompany the release that ships Phase 3.

## Consequences

- Existing user sessions **will be invalidated** when Phase 2
  ships (claim format change) and again if key material changes
  in Phase 3. Users must re-login.
- Deployers must generate and manage an RSA key pair (Phase 3).
  A development-mode auto-generated key reduces friction for
  local setups.
- Token revocation is **not** included in the initial scope but
  the architecture cleanly supports adding it later (Phase 4 /
  Option E).
- The centralised `jwt` module makes future algorithm changes
  (e.g., migrating to EdDSA) a localised, single-module change.
- External services can verify tokens using only the public key,
  enabling zero-trust verification without secret sharing.
