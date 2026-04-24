//! `RequirePermission<A>` extractor — ADR-T-008 Phase 2.
//!
//! Enforces role-based authorization at the HTTP boundary before the
//! handler runs.  Each handler declares the [`Action`] it requires via
//! a zero-sized marker type that implements [`ActionMarker`]:
//!
//! ```text
//! async fn add_category(
//!     RequirePermission(actor, _): RequirePermission<AddCategory>,
//!     // …
//! ) -> Response { … }
//! ```
//!
//! The extractor:
//!
//! 1. Extracts the bearer token (missing → `Guest`; invalid → 401).
//! 2. Resolves the caller's [`Role`] from the database.
//! 3. Checks the [`PermissionMatrix`](crate::services::authorization::PermissionMatrix).
//! 4. Rejects with 401 (`Guest` denied) or 403 (authenticated but
//!    insufficient role).
//! 5. On success, yields the resolved [`Actor`] to the handler.
//!
//! # `Actor` API
//!
//! The [`Actor`] struct provides three accessors for the caller’s
//! identity:
//!
//! - [`Actor::user_id()`] — panics on `Guest`; use only in handlers
//!   guarded by an action that denies `Guest`.
//! - [`Actor::try_user_id()`] — returns `Option<UserId>`, safe for
//!   any caller including guests.
//! - [`Actor::is_authenticated()`] — returns `true` for non-guest
//!   actors.
//!
//! # Compile-time safety
//!
//! The `action_markers!` macro emits a `const` assertion that the
//! number of marker structs equals `Action::ALL.len()`.  Adding an
//! `Action` variant without a corresponding marker (or vice versa) is
//! a compile error.

use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::Arc;

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};

use crate::common::AppData;
use crate::errors::AuthError;
use crate::models::user::UserId;
use crate::services::authorization::{Action, Role};
use crate::web::api::server::v1::extractors::bearer_token::BearerToken;

// ── Actor ────────────────────────────────────────────────────────────

/// Resolved caller identity after permission checking.
#[derive(Debug, Clone)]
pub struct Actor {
    pub user_id: Option<UserId>,
    pub role: Role,
}

impl Actor {
    /// Returns the user ID, or `None` for a guest actor.
    #[must_use]
    pub const fn try_user_id(&self) -> Option<UserId> {
        self.user_id
    }

    /// Returns the user ID.
    ///
    /// # Panics
    ///
    /// Panics if the actor is a guest.  Only call this in handlers
    /// guarded by an [`Action`] that denies `Guest` in the
    /// permission matrix.
    #[must_use]
    pub const fn user_id(&self) -> UserId {
        self.user_id
            .expect("BUG: called user_id() on a Guest actor — the handler's permission must deny Guest")
    }

    /// Returns `true` if the actor is an authenticated user (not a
    /// guest).
    #[must_use]
    pub const fn is_authenticated(&self) -> bool {
        self.user_id.is_some()
    }
}

// ── ActionMarker trait ───────────────────────────────────────────────

/// Associates a zero-sized type with an [`Action`] variant so
/// `RequirePermission<A>` can be generic over the required action.
pub trait ActionMarker: Send + Sync + 'static {
    const ACTION: Action;
}

macro_rules! action_markers {
    ($($variant:ident),* $(,)?) => {
        $(
            #[derive(Debug)]
            pub struct $variant;

            impl ActionMarker for $variant {
                const ACTION: Action = Action::$variant;
            }
        )*

        /// Compile-time check: one marker per `Action` variant.
        /// If `Action::ALL` gains or loses a variant without a
        /// matching update here, this static assert fires.
        const _: () = {
            const MARKER_COUNT: usize = 0 $(+ { let _ = Action::$variant; 1 })*;
            assert!(
                MARKER_COUNT == Action::ALL.len(),
                "action_markers! list is out of sync with Action::ALL",
            );
        };
    };
}

action_markers! {
    GetAboutPage,
    GetLicensePage,
    AddCategory,
    DeleteCategory,
    GetCategories,
    GetImageByUrl,
    GetSettingsSecret,
    GetPublicSettings,
    GetSiteName,
    AddTag,
    DeleteTag,
    GetTags,
    AddTorrent,
    GetTorrent,
    DeleteTorrent,
    GetTorrentInfo,
    GenerateTorrentInfoListing,
    ChangePassword,
    BanUser,
    GenerateUserProfileSpecification,
    UpdateTorrent,
    GetMyPermissions,
}

// ── RequirePermission extractor ──────────────────────────────────────

/// Axum extractor that enforces a permission check before the handler
/// runs.
///
/// `A` is a zero-sized marker type that carries the required [`Action`]
/// at the type level.
pub struct RequirePermission<A: ActionMarker>(pub Actor, pub PhantomData<A>);

impl<S, A> FromRequestParts<S> for RequirePermission<A>
where
    Arc<AppData>: FromRef<S>,
    S: Send + Sync,
    A: ActionMarker,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_data = Arc::from_ref(state);

        // 1. Resolve actor: no Authorization header → Guest; invalid → 401.
        let has_auth_header = parts.headers.contains_key("Authorization");

        let actor = if has_auth_header {
            let token = BearerToken::from_request_parts(parts, state).await?;

            let user_id = app_data
                .auth
                .get_user_id_from_bearer_token(token)
                .await
                .map_err(IntoResponse::into_response)?;

            let role = match app_data.user_repository.get_compact(&user_id).await {
                Ok(user) => {
                    if let Ok(role) = Role::from_str(&user.role) {
                        role
                    } else {
                        tracing::error!(user_id, role = %user.role, "unrecognised role in database — rejecting request");
                        return Err(AuthError::UnrecognisedRole.into_response());
                    }
                }
                Err(err) => {
                    // Preserve the underlying error category: only the
                    // genuine "user not found" case becomes a 404. DB
                    // / IO failures must surface as `DatabaseError`
                    // (500), not as a misleading 404 that masks the
                    // outage from operators.
                    if !matches!(err, crate::databases::database::Error::UserNotFound) {
                        tracing::error!(user_id, %err, "failed to load user for permission check");
                    }
                    return Err(AuthError::from(err).into_response());
                }
            };

            Actor {
                user_id: Some(user_id),
                role,
            }
        } else {
            Actor {
                user_id: None,
                role: Role::Guest,
            }
        };

        // 2. Check permission.
        if app_data.permissions.can(&actor.role, A::ACTION) {
            Ok(Self(actor, PhantomData))
        } else if actor.role == Role::Guest {
            Err(AuthError::UnauthorizedActionForGuests.into_response())
        } else {
            Err(AuthError::UnauthorizedAction.into_response())
        }
    }
}
