//! User services.
use std::collections::{HashMap, VecDeque};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use async_trait::async_trait;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
#[cfg(test)]
use mockall::automock;
use pbkdf2::password_hash::rand_core::OsRng;
use serde_derive::Deserialize;
use tracing::{debug, info};

use super::authentication::DbUserAuthenticationRepository;
use super::authorization::{self, ACTION};
use crate::config::{Configuration, PasswordConstraints};
use crate::databases::database::{Database, Error, UsersFilters, UsersSorting};
use crate::errors::ServiceError;
use crate::mailer::{password_fingerprint, ResetClaims, VerifyClaims};
use crate::models::response::UserProfilesResponse;
use crate::models::user::{UserCompact, UserId, UserProfile, Username};
use crate::services::authentication::verify_password;
use crate::utils::validation::validate_email_address;
use crate::web::api::server::v1::contexts::user::forms::{ChangePasswordForm, RegistrationForm};
use crate::{mailer, AsCSV};

/// Since user email could be optional, we need a way to represent "no email"
/// in the database. This function returns the string that should be used for
/// that purpose.
const fn no_email() -> String {
    String::new()
}

/// User request to generate a user profile listing.
#[derive(Debug, Deserialize)]
pub struct ListingRequest {
    /// Expects comma separated string
    pub filters: Option<String>,
    pub sort: Option<String>,
    pub page_size: Option<u8>,
    pub page: Option<u32>,
    pub search: Option<String>,
}

/// Internal specification for user profiles listings.
#[derive(Debug, Deserialize)]
pub struct ListingSpecification {
    pub offset: u64,
    /// Expects comma separated string
    pub filters: Option<Vec<UsersFilters>>,
    pub sort: Option<UsersSorting>,
    pub page_size: u8,
    pub search: Option<String>,
}

pub struct RegistrationService {
    configuration: Arc<Configuration>,
    mailer: Arc<mailer::Service>,
    user_repository: Arc<Box<dyn Repository>>,
}

impl RegistrationService {
    #[must_use]
    pub fn new(
        configuration: Arc<Configuration>,
        mailer: Arc<mailer::Service>,
        user_repository: Arc<Box<dyn Repository>>,
    ) -> Self {
        Self {
            configuration,
            mailer,
            user_repository,
        }
    }

    /// It registers a new user.
    ///
    /// # Errors
    ///
    /// This function will return a:
    ///
    /// * `ServiceError::EmailMissing` if email is required, but missing.
    /// * `ServiceError::EmailInvalid` if supplied email is badly formatted.
    /// * `ServiceError::PasswordsDontMatch` if the supplied passwords do not match.
    /// * `ServiceError::PasswordTooShort` if the supplied password is too short.
    /// * `ServiceError::PasswordTooLong` if the supplied password is too long.
    /// * `ServiceError::UsernameInvalid` if the supplied username is badly formatted.
    /// * `ServiceError::FailedToSendVerificationEmail` if unable to send the required verification email.
    /// * An error if unable to successfully hash the password.
    /// * An error if unable to insert user into the database.
    ///
    /// # Panics
    ///
    /// This function will panic if the email is required, but missing.
    //
    // TODO: registration itself is not rate-limited.  An attacker could
    // flood the system with throwaway accounts, filling the in-memory
    // `BoundedRateLimiter` buffers used by `PasswordResetService` and
    // `EmailVerificationService`.  A future change should add an IP-based
    // or CAPTCHA-gated registration rate limit.
    pub async fn register_user(&self, registration_form: &RegistrationForm, api_base_url: &str) -> Result<UserId, ServiceError> {
        info!("registering user: {}", registration_form.username);

        let settings = self.configuration.settings.read().await;

        let registration = match &settings.registration {
            Some(registration) => registration.clone(),
            None => {
                return Err(ServiceError::ClosedForRegistration);
            }
        };

        let password_constraints = PasswordConstraints {
            min_password_length: settings.auth.password_constraints.min_password_length,
            max_password_length: settings.auth.password_constraints.max_password_length,
        };
        drop(settings);

        let Ok(username) = registration_form.username.parse::<Username>() else {
            return Err(ServiceError::UsernameInvalid);
        };

        let opt_email = match &registration.email {
            Some(email) => {
                if email.required && registration_form.email.is_none() {
                    return Err(ServiceError::EmailMissing);
                }
                registration_form.email.as_ref().and_then(
                    |email| {
                        if email.trim().is_empty() {
                            None
                        } else {
                            Some(email.clone())
                        }
                    },
                )
            }
            None => None,
        };

        if let Some(email) = &opt_email {
            if !validate_email_address(email) {
                return Err(ServiceError::EmailInvalid);
            }
        }

        validate_password_constraints(
            &registration_form.password,
            &registration_form.confirm_password,
            &password_constraints,
        )?;

        let password_hash = hash_password(&registration_form.password)?;

        let user_id = self
            .user_repository
            .add(
                &username.to_string(),
                &opt_email.clone().unwrap_or(no_email()),
                &password_hash,
            )
            .await?;

        // If this is the first created account, give administrator rights
        if user_id == 1 {
            drop(self.user_repository.grant_admin_role(&user_id).await);
        }

        if let Some(email) = &registration.email {
            if email.verification_required {
                // Email verification is enabled
                if let Some(email) = opt_email {
                    let mail_res = self
                        .mailer
                        .send_verification_mail(&email, &registration_form.username, user_id, api_base_url)
                        .await;

                    if mail_res.is_err() {
                        drop(self.user_repository.delete(&user_id).await);
                        return Err(ServiceError::FailedToSendVerificationEmail);
                    }
                }
            }
        }

        Ok(user_id)
    }
}

pub struct ProfileService {
    configuration: Arc<Configuration>,
    user_authentication_repository: Arc<DbUserAuthenticationRepository>,
    authorization_service: Arc<authorization::Service>,
}

impl ProfileService {
    #[must_use]
    pub const fn new(
        configuration: Arc<Configuration>,
        user_repository: Arc<DbUserAuthenticationRepository>,
        authorization_service: Arc<authorization::Service>,
    ) -> Self {
        Self {
            configuration,
            user_authentication_repository: user_repository,
            authorization_service,
        }
    }

    /// It registers a new user.
    ///
    /// # Errors
    ///
    /// This function will return a:
    ///
    /// * `ServiceError::InvalidPassword` if the current password supplied is invalid.
    /// * `ServiceError::PasswordsDontMatch` if the supplied passwords do not match.
    /// * `ServiceError::PasswordTooShort` if the supplied password is too short.
    /// * `ServiceError::PasswordTooLong` if the supplied password is too long.
    /// * An error if unable to successfully hash the password.
    /// * An error if unable to change the password in the database.
    /// * An error if it is not possible to authorize the action
    pub async fn change_password(
        &self,
        maybe_user_id: Option<UserId>,
        change_password_form: &ChangePasswordForm,
    ) -> Result<(), ServiceError> {
        let Some(user_id) = maybe_user_id else {
            return Err(ServiceError::UnauthorizedActionForGuests);
        };

        self.authorization_service
            .authorize(ACTION::ChangePassword, maybe_user_id)
            .await?;

        info!("changing user password for user ID: {}", user_id);

        let settings = self.configuration.settings.read().await;

        let user_authentication = self
            .user_authentication_repository
            .get_user_authentication_from_id(&user_id)
            .await?;

        verify_password(change_password_form.current_password.as_bytes(), &user_authentication)?;

        let password_constraints = PasswordConstraints {
            min_password_length: settings.auth.password_constraints.min_password_length,
            max_password_length: settings.auth.password_constraints.max_password_length,
        };
        drop(settings);

        validate_password_constraints(
            &change_password_form.password,
            &change_password_form.confirm_password,
            &password_constraints,
        )?;

        let password_hash = hash_password(&change_password_form.password)?;

        self.user_authentication_repository
            .change_password(user_id, &password_hash)
            .await?;

        Ok(())
    }
}

pub struct BanService {
    user_profile_repository: Arc<DbUserProfileRepository>,
    banned_user_list: Arc<DbBannedUserList>,
    authorization_service: Arc<authorization::Service>,
}

impl BanService {
    #[must_use]
    pub const fn new(
        user_profile_repository: Arc<DbUserProfileRepository>,
        banned_user_list: Arc<DbBannedUserList>,
        authorization_service: Arc<authorization::Service>,
    ) -> Self {
        Self {
            user_profile_repository,
            banned_user_list,
            authorization_service,
        }
    }

    /// Ban a user from the Index.
    ///
    /// # Errors
    ///
    /// This function will return a:
    ///
    /// * `ServiceError::InternalServerError` if unable get user from the request.
    /// * An error if unable to get user profile from supplied username.
    /// * An error if unable to set the ban of the user in the database.
    pub async fn ban_user(&self, username_to_be_banned: &str, maybe_user_id: Option<UserId>) -> Result<(), ServiceError> {
        let Some(user_id) = maybe_user_id else {
            return Err(ServiceError::UnauthorizedActionForGuests);
        };

        self.authorization_service.authorize(ACTION::BanUser, maybe_user_id).await?;

        debug!("user with ID {} banning username: {username_to_be_banned}", user_id);

        let user_profile = self
            .user_profile_repository
            .get_user_profile_from_username(username_to_be_banned)
            .await?;

        self.banned_user_list.add(&user_profile.user_id).await?;

        Ok(())
    }
}

pub struct ListingService {
    configuration: Arc<Configuration>,
    user_profile_repository: Arc<DbUserProfileRepository>,
    authorization_service: Arc<authorization::Service>,
}

impl ListingService {
    #[must_use]
    pub const fn new(
        configuration: Arc<Configuration>,
        user_profile_repository: Arc<DbUserProfileRepository>,
        authorization_service: Arc<authorization::Service>,
    ) -> Self {
        Self {
            configuration,
            user_profile_repository,
            authorization_service,
        }
    }

    /// It converts the user listing request into an internal listing specification.
    ///    
    /// # Errors
    ///
    /// Returns a `ServiceError::InvalidUserListing` if there is an incorrect value in the url params for the listing request.
    pub async fn listing_specification_from_user_request(
        &self,
        maybe_user_id: Option<UserId>,
        request: &ListingRequest,
    ) -> Result<ListingSpecification, ServiceError> {
        self.authorization_service
            .authorize(ACTION::GenerateUserProfileSpecification, maybe_user_id)
            .await?;

        let settings = self.configuration.settings.read().await;
        let default_user_profile_page_size = settings.api.default_user_profile_page_size;
        let max_user_profile_page_size = settings.api.max_user_profile_page_size;
        drop(settings);

        let page = request.page.unwrap_or(0);
        let page_size = request.page_size.unwrap_or(default_user_profile_page_size);

        // Guard that page size does not exceed the maximum
        let page_size = if page_size > max_user_profile_page_size {
            max_user_profile_page_size
        } else {
            page_size
        };

        let offset = u64::from(page * u32::from(page_size));

        let sort = match &request.sort {
            Some(sort_value) => Some(UsersSorting::from_str(sort_value).map_err(|_| ServiceError::InvalidUserListing)?),
            None => None,
        };

        let filter_values = request
            .filters
            .as_csv::<String>()
            .map_err(|()| ServiceError::InvalidUserListing)?;

        let filters = if let Some(filter_values) = filter_values {
            let mut sanitized_filters: Vec<UsersFilters> = Vec::new();
            for filter in filter_values {
                match filter.as_str() {
                    "TorrentUploader" => sanitized_filters
                        .push(UsersFilters::from_str("TorrentUploader").map_err(|_| ServiceError::InvalidUserListing)?),
                    "EmailNotVerified" => sanitized_filters
                        .push(UsersFilters::from_str("EmailNotVerified").map_err(|_| ServiceError::InvalidUserListing)?),
                    "EmailVerified" => sanitized_filters
                        .push(UsersFilters::from_str("EmailVerified").map_err(|_| ServiceError::InvalidUserListing)?),
                    _ => return Err(ServiceError::InvalidUserListing),
                }
            }
            Some(sanitized_filters)
        } else {
            None
        };

        Ok(ListingSpecification {
            offset,
            filters,
            sort,
            page_size,
            search: request.search.clone(),
        })
    }

    /// Returns a list of all the user profiles matching the search criteria.
    ///
    /// # Errors
    ///
    /// Returns a `ServiceError::DatabaseError` if the database query fails.
    pub async fn generate_user_profile_listing(
        &self,
        listing: &ListingSpecification,
    ) -> Result<UserProfilesResponse, ServiceError> {
        let user_profiles_response = self.user_profile_repository.generate_listing(listing).await?;

        Ok(user_profiles_response)
    }
}

/// Per-key state tracked by the [`BoundedRateLimiter`].
struct ThrottleEntry {
    /// Number of requests recorded since the last reset.
    count: u32,
    /// When the most recent request was recorded.
    last_issued: Instant,
}

/// Result of a rate-limit check performed by [`BoundedRateLimiter::check`].
enum ThrottleStatus {
    /// The request is allowed.
    Allowed,
    /// The request is rate-limited; the caller must wait this many more seconds.
    Limited { remaining_secs: u64 },
    /// The hard attempt cap has been reached.
    HardLocked,
}

/// A bounded, ring-buffer-backed rate limiter.
///
/// Entries are indexed by a `HashMap` for O(1) lookup and stored in insertion
/// order in a `VecDeque` so that the oldest entry can be evicted in O(1) when
/// the buffer is full.  A periodic sweep removes entries whose
/// `last_issued` is older than a configurable staleness threshold.
///
/// This is a general-purpose structure shared by [`PasswordResetService`] and
/// [`EmailVerificationService`].
struct BoundedRateLimiter {
    /// Maps lowercase key → logical index into `entries`.
    index: HashMap<String, usize>,
    /// Ring of `(key, state)` pairs in insertion order.
    entries: VecDeque<(String, ThrottleEntry)>,
    /// Monotonically increasing generation counter used as the logical index
    /// stored in `index`.  The physical position in `entries` is
    /// `logical - base_generation`.
    generation: usize,
    /// Generation value of the front of the deque.
    base_generation: usize,
}

impl BoundedRateLimiter {
    /// Hard upper bound on the number of tracked emails.
    const CAPACITY: usize = 50_000;

    fn new() -> Self {
        Self {
            index: HashMap::new(),
            entries: VecDeque::new(),
            generation: 0,
            base_generation: 0,
        }
    }

    /// Returns an immutable reference to the state for `key`, if present.
    ///
    /// The key is lowercased before lookup so callers do not need to
    /// normalise it themselves.
    fn get(&self, key: &str) -> Option<&ThrottleEntry> {
        let normalized = key.to_lowercase();
        let &slot_gen = self.index.get(&normalized)?;
        let pos = slot_gen - self.base_generation;
        self.entries.get(pos).map(|(_, state)| state)
    }

    /// Checks the exponential-backoff rate limit for `key` against `policy`.
    fn check(&self, key: &str, policy: &crate::config::ThrottlePolicy) -> ThrottleStatus {
        if let Some(entry) = self.get(key) {
            if entry.count >= policy.max_attempts {
                return ThrottleStatus::HardLocked;
            }
            if entry.count > 0 {
                let shift = (entry.count - 1).min(63);
                let backoff_secs = policy
                    .base_backoff_secs
                    .saturating_mul(1u64 << shift)
                    .min(policy.max_backoff_secs);
                let elapsed = entry.last_issued.elapsed().as_secs();
                if elapsed < backoff_secs {
                    return ThrottleStatus::Limited {
                        remaining_secs: backoff_secs - elapsed,
                    };
                }
            }
        }
        ThrottleStatus::Allowed
    }

    /// Records an attempt for `email`.  If the email already has an entry its
    /// count is incremented in-place; otherwise a new entry is pushed to the
    /// back of the ring, evicting the oldest entry if the buffer is full.
    ///
    /// A staleness sweep is run first so that expired entries are reclaimed
    /// before we consider evicting by age.
    fn record(&mut self, email: &str, max_staleness_secs: u64) {
        self.sweep_stale(max_staleness_secs);

        let key = email.to_lowercase();

        // Fast path: entry already exists — update in place.
        if let Some(&slot_gen) = self.index.get(&key) {
            let pos = slot_gen - self.base_generation;
            if let Some((_, state)) = self.entries.get_mut(pos) {
                state.count += 1;
                state.last_issued = Instant::now();
                return;
            }
        }

        // Evict the oldest entry if we are at capacity.
        if self.entries.len() >= Self::CAPACITY {
            self.evict_oldest();
        }

        let slot_gen = self.generation;
        self.generation += 1;
        self.entries.push_back((
            key.clone(),
            ThrottleEntry {
                count: 1,
                last_issued: Instant::now(),
            },
        ));
        self.index.insert(key, slot_gen);
    }

    /// Removes the entry for `email`, if present.
    fn remove(&mut self, email: &str) {
        let key = email.to_lowercase();
        if let Some(slot_gen) = self.index.remove(&key) {
            let pos = slot_gen - self.base_generation;
            if let Some((k, _)) = self.entries.get_mut(pos) {
                // Mark the slot as tombstoned by clearing the key so the
                // sweep/evict path skips the index removal.
                k.clear();
            }
        }
    }

    /// Evicts entries from the front of the deque that are older than
    /// `max_staleness_secs`.  Tombstoned entries (empty key) are also removed.
    fn sweep_stale(&mut self, max_staleness_secs: u64) {
        let threshold = Duration::from_secs(max_staleness_secs);
        while let Some((key, state)) = self.entries.front() {
            if key.is_empty() || state.last_issued.elapsed() >= threshold {
                let (evicted_key, _) = self.entries.pop_front().expect("front exists");
                if !evicted_key.is_empty() {
                    self.index.remove(&evicted_key);
                }
                self.base_generation += 1;
            } else {
                break;
            }
        }
    }

    /// Evicts the single oldest entry from the front of the deque.
    fn evict_oldest(&mut self) {
        if let Some((evicted_key, _)) = self.entries.pop_front() {
            if !evicted_key.is_empty() {
                self.index.remove(&evicted_key);
            }
            self.base_generation += 1;
        }
    }
}

pub struct PasswordResetService {
    configuration: Arc<Configuration>,
    mailer: Arc<mailer::Service>,
    user_profile_repository: Arc<DbUserProfileRepository>,
    user_authentication_repository: Arc<DbUserAuthenticationRepository>,
    authorization_service: Arc<authorization::Service>,
    /// Bounded, in-memory per-email rate limiter for password reset requests.
    rate_limiter: RwLock<BoundedRateLimiter>,
}

impl PasswordResetService {
    /// Minimum time the `send_reset_link` handler will take before returning,
    /// regardless of the code-path. This prevents timing-based account
    /// enumeration.
    const MIN_RESET_RESPONSE: Duration = Duration::from_secs(1);

    #[must_use]
    pub fn new(
        configuration: Arc<Configuration>,
        mailer: Arc<mailer::Service>,
        user_profile_repository: Arc<DbUserProfileRepository>,
        user_authentication_repository: Arc<DbUserAuthenticationRepository>,
        authorization_service: Arc<authorization::Service>,
    ) -> Self {
        Self {
            configuration,
            mailer,
            user_profile_repository,
            user_authentication_repository,
            authorization_service,
            rate_limiter: RwLock::new(BoundedRateLimiter::new()),
        }
    }

    /// Sends a password reset link to the email if it belongs to a verified user.
    ///
    /// To prevent account enumeration, this method always returns `Ok(())`
    /// regardless of whether the email exists or is verified.
    ///
    /// Admins bypass the rate limiter so they can trigger a reset email on
    /// behalf of locked-out users.
    ///
    /// # Errors
    ///
    /// This function will return a:
    ///
    /// * An authorization error if the action is not permitted.
    /// * `ServiceError::PasswordResetLocked` if the email is rate-limited.
    /// * `ServiceError::FailedToSendResetPassword` if unable to send the email.
    pub async fn send_reset_link(
        &self,
        maybe_user_id: Option<UserId>,
        email: &str,
        api_base_url: &str,
    ) -> Result<(), ServiceError> {
        // Authorization and rate-limit checks run *before* the constant-time
        // window because their errors are returned to the caller anyway (no
        // information leak).
        self.authorization_service
            .authorize(ACTION::SendPasswordResetLink, maybe_user_id)
            .await?;

        // Admins bypass the rate limit so they can send reset links on behalf
        // of locked-out users.
        let is_admin = self
            .authorization_service
            .authorize(ACTION::IsAdmin, maybe_user_id)
            .await
            .is_ok();

        // Logged-in users resetting their own password also bypass the rate
        // limit — they have already proven their identity by being
        // authenticated.  We compare emails case-insensitively.
        let is_own_email = if let Some(user_id) = maybe_user_id {
            self.user_profile_repository
                .get_user_profile_from_id(user_id)
                .await
                .is_ok_and(|p| p.email.eq_ignore_ascii_case(email))
        } else {
            false
        };

        if !is_admin && !is_own_email {
            let settings = self.configuration.settings.read().await;
            let policy = settings.auth.password_reset_policy.clone();
            drop(settings);
            self.check_rate_limit(email, &policy)?;
        }

        // Run the real work alongside a minimum-duration timer so that the
        // response always takes at least this floor regardless of whether
        // the email exists, is verified, or the SMTP send succeeds. This
        // prevents an attacker from enumerating valid accounts via response
        // time.
        let real_work = self.send_reset_link_inner(email, api_base_url);
        let floor = tokio::time::sleep(Self::MIN_RESET_RESPONSE);

        let (result, ()) = tokio::join!(real_work, floor);

        result
    }

    /// Inner logic for [`Self::send_reset_link`], factored out so the caller
    /// can enforce a constant-time floor around it.
    async fn send_reset_link_inner(&self, email: &str, api_base_url: &str) -> Result<(), ServiceError> {
        // Look up the user by email. If not found or not verified, silently
        // succeed to prevent account enumeration.
        let Ok(user_profile) = self.user_profile_repository.get_user_profile_from_email(email).await else {
            info!("Password reset requested for unknown email (not revealing to caller)");
            return Ok(());
        };

        if !user_profile.email_verified {
            info!("Password reset requested for unverified email (not revealing to caller)");
            return Ok(());
        }

        // Fetch the current password hash so the token self-invalidates when
        // the password is changed before it is used.
        let user_auth = self
            .user_authentication_repository
            .get_user_authentication_from_id(&user_profile.user_id)
            .await
            .map_err(|_| ServiceError::InternalServerError)?;

        info!("Sending password reset link for user: {}", user_profile.username);

        self.mailer
            .send_reset_password_mail(
                email,
                &user_profile.username,
                user_profile.user_id,
                &user_auth.password_hash,
                api_base_url,
            )
            .await?;

        // Record the attempt *after* successfully sending the email so that a
        // transient SMTP failure does not consume an attempt.
        self.record_attempt(email);

        Ok(())
    }

    /// Completes a password reset using a JWT token from the reset link.
    ///
    /// This method is intentionally **unauthenticated**: the caller does not
    /// need to be logged in.  Identity is established through the signed,
    /// time-limited JWT that was emailed to the account owner.  The embedded
    /// password-hash fingerprint additionally ensures the token is single-use.
    ///
    /// On success the rate-limiter state for the user's email is cleared.
    ///
    /// # Errors
    ///
    /// This function will return a:
    ///
    /// * `ServiceError::TokenInvalid` if the token is invalid, not a
    ///   password-reset token, or the password has already been changed since
    ///   the token was issued.
    /// * `ServiceError::PasswordsDontMatch` if the passwords don't match.
    /// * `ServiceError::PasswordTooShort` / `PasswordTooLong` if constraints are violated.
    /// * Database or hashing errors.
    pub async fn complete_reset(&self, token: &str, password: &str, confirm_password: &str) -> Result<(), ServiceError> {
        let settings = self.configuration.settings.read().await;

        let token_data = decode::<ResetClaims>(
            token,
            &DecodingKey::from_secret(settings.auth.user_claim_token_pepper.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| ServiceError::TokenInvalid)?;

        if token_data.claims.iss != "password-reset" {
            return Err(ServiceError::TokenInvalid);
        }

        let password_constraints = PasswordConstraints {
            min_password_length: settings.auth.password_constraints.min_password_length,
            max_password_length: settings.auth.password_constraints.max_password_length,
        };
        drop(settings);

        let user_id = token_data.claims.sub;

        // Verify the password fingerprint matches the current hash. This
        // ensures the token is single-use: once the password is changed the
        // fingerprint no longer matches and the token is rejected.
        let current_auth = self
            .user_authentication_repository
            .get_user_authentication_from_id(&user_id)
            .await
            .map_err(|_| ServiceError::TokenInvalid)?;

        if token_data.claims.pwd != password_fingerprint(&current_auth.password_hash) {
            return Err(ServiceError::TokenInvalid);
        }

        validate_password_constraints(password, confirm_password, &password_constraints)?;

        let password_hash = hash_password(password)?;

        self.user_authentication_repository
            .change_password(user_id, &password_hash)
            .await?;

        // Clear the rate-limiter state for this user so they start fresh.
        self.clear_rate_limit_for_user(user_id).await;

        Ok(())
    }

    /// Checks the backoff-based rate limit for password reset requests.
    ///
    /// The required wait time between requests grows as
    /// `min(base_backoff_secs * 2^(count - 1), max_backoff_secs)` where
    /// `count` is the number of resets already issued since the last
    /// successful password change.
    ///
    /// After `max_attempts` the account is hard-locked until the password is
    /// successfully changed (or an admin triggers a reset on behalf of the
    /// user).
    ///
    /// # Errors
    ///
    /// Returns `ServiceError::PasswordResetLocked { remaining_secs }` if the
    /// email must wait before another reset can be issued.
    fn check_rate_limit(&self, email: &str, policy: &crate::config::ThrottlePolicy) -> Result<(), ServiceError> {
        let limiter = self.rate_limiter.read().expect("rate-limit lock poisoned");
        match limiter.check(email, policy) {
            ThrottleStatus::Allowed => Ok(()),
            ThrottleStatus::Limited { remaining_secs } => Err(ServiceError::PasswordResetLocked { remaining_secs }),
            ThrottleStatus::HardLocked => Err(ServiceError::PasswordResetMaxAttemptsReached),
        }
    }

    /// Records a successful email send for rate-limiting purposes.
    ///
    /// Also triggers a staleness sweep so that entries older than
    /// `max_backoff_secs` are evicted before we consider capacity.
    fn record_attempt(&self, email: &str) {
        // We need `max_backoff_secs` for the staleness sweep.  Because this
        // is called from a sync context we cannot `.await` on the tokio
        // `RwLock` that guards the config.  Falling back to the default
        // policy value is safe: in the worst case we sweep a little too
        // aggressively or conservatively.
        let max_staleness_secs = crate::config::ThrottlePolicy::default().max_backoff_secs;

        let mut limiter = self.rate_limiter.write().expect("rate-limit lock poisoned");
        limiter.record(email, max_staleness_secs);
        drop(limiter);
    }

    /// Clears rate-limit state for a given user after a successful password
    /// change. Looks up the email by user ID.
    async fn clear_rate_limit_for_user(&self, user_id: UserId) {
        let Ok(profile) = self.user_profile_repository.get_user_profile_from_id(user_id).await else {
            return;
        };
        let mut limiter = self.rate_limiter.write().expect("rate-limit lock poisoned");
        limiter.remove(&profile.email);
        drop(limiter);
    }
}

/// Service for email verification, including resending verification links
/// with rate limiting that mirrors [`PasswordResetService`].
///
/// The flow is:
///
/// 1. **Initial send** — handled by [`RegistrationService::register_user`] (no
///    rate limiting; one email per registration).
/// 2. **Resend** — [`EmailVerificationService::resend_verification_link`]
///    (rate limited, constant-time, anti-enumeration).
/// 3. **Verify** — [`EmailVerificationService::verify_email`] (consumes the
///    token and clears rate-limiter state).
pub struct EmailVerificationService {
    configuration: Arc<Configuration>,
    mailer: Arc<mailer::Service>,
    user_profile_repository: Arc<DbUserProfileRepository>,
    authorization_service: Arc<authorization::Service>,
    /// Bounded, in-memory per-email rate limiter for verification resend requests.
    rate_limiter: RwLock<BoundedRateLimiter>,
}

impl EmailVerificationService {
    /// Minimum time the `resend_verification_link` handler will take before
    /// returning, regardless of the code-path.  This prevents timing-based
    /// account enumeration.
    const MIN_RESPONSE: Duration = Duration::from_secs(1);

    #[must_use]
    pub fn new(
        configuration: Arc<Configuration>,
        mailer: Arc<mailer::Service>,
        user_profile_repository: Arc<DbUserProfileRepository>,
        authorization_service: Arc<authorization::Service>,
    ) -> Self {
        Self {
            configuration,
            mailer,
            user_profile_repository,
            authorization_service,
            rate_limiter: RwLock::new(BoundedRateLimiter::new()),
        }
    }

    /// Resends a verification link to `email` if it belongs to an unverified
    /// user.
    ///
    /// To prevent account enumeration, this method always returns `Ok(())`
    /// regardless of whether the email exists or is already verified.
    ///
    /// Admins bypass the rate limiter so they can trigger a verification email
    /// on behalf of locked-out users.
    ///
    /// # Errors
    ///
    /// This function will return:
    ///
    /// * An authorization error if the action is not permitted.
    /// * `ServiceError::VerificationResendLocked` if the email is rate-limited.
    /// * `ServiceError::FailedToSendVerificationEmail` if unable to send the
    ///   email.
    pub async fn resend_verification_link(
        &self,
        maybe_user_id: Option<UserId>,
        email: &str,
        api_base_url: &str,
    ) -> Result<(), ServiceError> {
        self.authorization_service
            .authorize(ACTION::ResendVerificationLink, maybe_user_id)
            .await?;

        let is_admin = self
            .authorization_service
            .authorize(ACTION::IsAdmin, maybe_user_id)
            .await
            .is_ok();

        if !is_admin {
            let settings = self.configuration.settings.read().await;
            let policy = settings.auth.email_verification_policy.clone();
            drop(settings);
            self.check_rate_limit(email, &policy)?;
        }

        let real_work = self.resend_verification_link_inner(email, api_base_url);
        let floor = tokio::time::sleep(Self::MIN_RESPONSE);

        let (result, ()) = tokio::join!(real_work, floor);

        result
    }

    /// Inner logic for [`Self::resend_verification_link`], factored out so
    /// the caller can enforce a constant-time floor around it.
    async fn resend_verification_link_inner(&self, email: &str, api_base_url: &str) -> Result<(), ServiceError> {
        let Ok(user_profile) = self.user_profile_repository.get_user_profile_from_email(email).await else {
            info!("Verification resend requested for unknown email (not revealing to caller)");
            return Ok(());
        };

        if user_profile.email_verified {
            info!("Verification resend requested for already-verified email (not revealing to caller)");
            return Ok(());
        }

        info!("Resending verification email for user: {}", user_profile.username);

        self.mailer
            .send_verification_mail(email, &user_profile.username, user_profile.user_id, api_base_url)
            .await?;

        self.record_attempt(email);

        Ok(())
    }

    /// Verifies the email address of a user via the token sent to the user's
    /// email.
    ///
    /// On success the rate-limiter state for the user's email is cleared.
    ///
    /// # Errors
    ///
    /// This function will return a `ServiceError::DatabaseError` if unable to
    /// update the user's email verification status.
    pub async fn verify_email(&self, token: &str) -> Result<bool, ServiceError> {
        let settings = self.configuration.settings.read().await;

        let token_data = match decode::<VerifyClaims>(
            token,
            &DecodingKey::from_secret(settings.auth.user_claim_token_pepper.as_bytes()),
            &Validation::new(Algorithm::HS256),
        ) {
            Ok(token_data) => {
                if !token_data.claims.iss.eq("email-verification") {
                    return Ok(false);
                }

                token_data.claims
            }
            Err(_) => return Ok(false),
        };

        drop(settings);

        let user_id = token_data.sub;

        if self.user_profile_repository.verify_email(&user_id).await.is_err() {
            return Err(ServiceError::DatabaseError);
        }

        // Clear the rate-limiter state so subsequent resend requests start
        // fresh.
        self.clear_rate_limit_for_user(user_id).await;

        Ok(true)
    }

    /// Checks the backoff-based rate limit for verification resend requests.
    fn check_rate_limit(&self, email: &str, policy: &crate::config::ThrottlePolicy) -> Result<(), ServiceError> {
        let limiter = self.rate_limiter.read().expect("rate-limit lock poisoned");
        match limiter.check(email, policy) {
            ThrottleStatus::Allowed => Ok(()),
            ThrottleStatus::Limited { remaining_secs } => Err(ServiceError::VerificationResendLocked { remaining_secs }),
            ThrottleStatus::HardLocked => Err(ServiceError::VerificationResendMaxAttemptsReached),
        }
    }

    /// Records a successful email send for rate-limiting purposes.
    fn record_attempt(&self, email: &str) {
        let max_staleness_secs = crate::config::ThrottlePolicy::default().max_backoff_secs;

        let mut limiter = self.rate_limiter.write().expect("rate-limit lock poisoned");
        limiter.record(email, max_staleness_secs);
        drop(limiter);
    }

    /// Clears rate-limit state for a given user after a successful email
    /// verification.  Looks up the email by user ID.
    async fn clear_rate_limit_for_user(&self, user_id: UserId) {
        let Ok(profile) = self.user_profile_repository.get_user_profile_from_id(user_id).await else {
            return;
        };
        let mut limiter = self.rate_limiter.write().expect("rate-limit lock poisoned");
        limiter.remove(&profile.email);
        drop(limiter);
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Repository: Sync + Send {
    async fn get_compact(&self, user_id: &UserId) -> Result<UserCompact, ServiceError>;
    async fn grant_admin_role(&self, user_id: &UserId) -> Result<(), Error>;
    async fn delete(&self, user_id: &UserId) -> Result<(), Error>;
    async fn add(&self, username: &str, email: &str, password_hash: &str) -> Result<UserId, Error>;
}

pub struct DbUserRepository {
    database: Arc<Box<dyn Database>>,
}

impl DbUserRepository {
    #[must_use]
    pub fn new(database: Arc<Box<dyn Database>>) -> Self {
        Self { database }
    }
}

#[async_trait]
impl Repository for DbUserRepository {
    /// It returns the compact user.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    async fn get_compact(&self, user_id: &UserId) -> Result<UserCompact, ServiceError> {
        // todo: persistence layer should have its own errors instead of
        // returning a `ServiceError`.
        self.database
            .get_user_compact_from_id(*user_id)
            .await
            .map_err(|_| ServiceError::UserNotFound)
    }

    /// It grants the admin role to the user.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    async fn grant_admin_role(&self, user_id: &UserId) -> Result<(), Error> {
        self.database.grant_admin_role(*user_id).await
    }

    /// It deletes the user.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    async fn delete(&self, user_id: &UserId) -> Result<(), Error> {
        self.database.delete_user(*user_id).await
    }

    /// It adds a new user.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    async fn add(&self, username: &str, email: &str, password_hash: &str) -> Result<UserId, Error> {
        self.database.insert_user_and_get_id(username, email, password_hash).await
    }
}

pub struct DbUserProfileRepository {
    database: Arc<Box<dyn Database>>,
}

impl DbUserProfileRepository {
    #[must_use]
    pub fn new(database: Arc<Box<dyn Database>>) -> Self {
        Self { database }
    }

    /// It marks the user's email as verified.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn verify_email(&self, user_id: &UserId) -> Result<(), Error> {
        self.database.verify_email(*user_id).await
    }

    /// It get the user profile from the username.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn get_user_profile_from_username(&self, username: &str) -> Result<UserProfile, Error> {
        self.database.get_user_profile_from_username(username).await
    }

    /// It get the user profile from the email address.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn get_user_profile_from_email(&self, email: &str) -> Result<UserProfile, Error> {
        self.database.get_user_profile_from_email(email).await
    }

    /// It gets the user profile from a user ID.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn get_user_profile_from_id(&self, user_id: UserId) -> Result<UserProfile, Error> {
        self.database.get_user_profile_from_id(user_id).await
    }

    /// It gets all the user profiles for all the users.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn generate_listing(&self, specification: &ListingSpecification) -> Result<UserProfilesResponse, Error> {
        self.database
            .get_user_profiles_search_paginated(
                &specification.search,
                &specification.filters,
                specification.sort,
                specification.offset,
                specification.page_size,
            )
            .await
    }
}

pub struct DbBannedUserList {
    database: Arc<Box<dyn Database>>,
}

impl DbBannedUserList {
    #[must_use]
    pub fn new(database: Arc<Box<dyn Database>>) -> Self {
        Self { database }
    }

    /// It add a user to the banned users list.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    ///
    /// # Panics
    ///
    /// It panics if the expiration date cannot be parsed. It should never
    /// happen as the date is hardcoded for now.
    pub async fn add(&self, user_id: &UserId) -> Result<(), Error> {
        // todo: add reason and `date_expiry` parameters to request.

        // code-review: add the user ID of the user who banned the user.

        // For the time being, we will not use a reason for banning a user.
        let reason = "no reason".to_string();

        // User will be banned until the year 9999
        let date_expiry = chrono::NaiveDateTime::parse_from_str("9999-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
            .expect("Could not parse date from 9999-01-01 00:00:00.");

        self.database.ban_user(*user_id, &reason, date_expiry).await
    }
}

fn validate_password_constraints(
    password: &str,
    confirm_password: &str,
    password_rules: &PasswordConstraints,
) -> Result<(), ServiceError> {
    if password != confirm_password {
        return Err(ServiceError::PasswordsDontMatch);
    }

    let password_length = password.len();

    if password_length < password_rules.min_password_length {
        return Err(ServiceError::PasswordTooShort);
    }

    if password_length > password_rules.max_password_length {
        return Err(ServiceError::PasswordTooLong);
    }

    Ok(())
}

fn hash_password(password: &str) -> Result<String, ServiceError> {
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();

    Ok(password_hash)
}
