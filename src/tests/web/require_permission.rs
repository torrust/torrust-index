//! Crate tests for the `RequirePermission<A>` extractor
//! (`src/web/api/server/v1/extractors/require_permission.rs`).
//!
//! Exercises the full extraction path: token parsing → role lookup →
//! matrix check → 401/403/200 response (ADR-T-008 Phase 2).
//!
//! # Test index
//!
//! ## Actor
//!
//! - [`actor_user_id_returns_value_for_authenticated`] — `user_id()`
//!   returns the stored `UserId` for an authenticated actor.
//! - [`actor_user_id_panics_for_guest`] — `user_id()` panics when
//!   called on a `Guest` actor.
//! - [`actor_try_user_id_returns_some_for_authenticated`] —
//!   `try_user_id()` returns `Some` for an authenticated actor.
//! - [`actor_try_user_id_returns_none_for_guest`] — `try_user_id()`
//!   returns `None` for a `Guest` actor.
//! - [`actor_is_authenticated_returns_true_for_user`] —
//!   `is_authenticated()` returns `true` for an authenticated actor.
//! - [`actor_is_authenticated_returns_false_for_guest`] —
//!   `is_authenticated()` returns `false` for a `Guest` actor.
//!
//! ## `RequirePermission` extractor (full path)
//!
//! - [`guest_denied_action_returns_401`] — no `Authorization` header +
//!   action denied for `Guest` → `401 Unauthorized`.
//! - [`guest_allowed_action_returns_200`] — no `Authorization` header +
//!   action allowed for `Guest` → `200 OK`.
//! - [`authenticated_denied_action_returns_403`] — valid token with
//!   `Registered` role + admin-only action → `403 Forbidden`.
//! - [`authenticated_allowed_action_returns_200`] — valid token with
//!   `Registered` role + allowed action → `200 OK`.
//! - [`admin_allowed_for_all_actions`] — valid token with `Admin`
//!   role + any action → `200 OK`.

use std::sync::Arc;

use axum::Router;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use hyper::StatusCode;
use tower::ServiceExt as _;

use crate::cache::image::manager::ImageCacheService;
use crate::common::AppData;
use crate::config::Configuration;
use crate::databases::database;
use crate::models::user::UserId;
use crate::services::authentication::{DbUserAuthenticationRepository, JsonWebToken, Service as AuthenticationService};
use crate::services::authorization::{PermissionMatrix, Permissions, Role};
use crate::services::category::{self, DbCategoryRepository};
use crate::services::tag::{self, DbTagRepository};
use crate::services::torrent::{
    DbCanonicalInfoHashGroupRepository, DbTorrentAnnounceUrlRepository, DbTorrentFileRepository, DbTorrentInfoRepository,
    DbTorrentListingGenerator, DbTorrentRepository, DbTorrentTagRepository,
};
use crate::services::user::{self, DbBannedUserList, DbUserProfileRepository, DbUserRepository, Repository};
use crate::services::{about, proxy, settings, torrent};
use crate::tracker::statistics_importer::StatisticsImporter;
use crate::web::api::server::v1::auth::Authentication;
use crate::web::api::server::v1::extractors::require_permission::{Actor, AddCategory, GetCategories, RequirePermission};
use crate::{mailer, tracker};

// ── Actor ────────────────────────────────────────────────────────────

#[test]
fn actor_user_id_returns_value_for_authenticated() {
    let actor = Actor {
        user_id: Some(42),
        role: Role::Registered,
    };
    assert_eq!(actor.user_id(), 42);
}

#[test]
#[should_panic(expected = "BUG: called user_id() on a Guest actor")]
fn actor_user_id_panics_for_guest() {
    let actor = Actor {
        user_id: None,
        role: Role::Guest,
    };
    let _ = actor.user_id();
}

#[test]
fn actor_try_user_id_returns_some_for_authenticated() {
    let actor = Actor {
        user_id: Some(42),
        role: Role::Registered,
    };
    assert_eq!(actor.try_user_id(), Some(42));
}

#[test]
fn actor_try_user_id_returns_none_for_guest() {
    let actor = Actor {
        user_id: None,
        role: Role::Guest,
    };
    assert_eq!(actor.try_user_id(), None);
}

#[test]
fn actor_is_authenticated_returns_true_for_user() {
    let actor = Actor {
        user_id: Some(1),
        role: Role::Registered,
    };
    assert!(actor.is_authenticated());
}

#[test]
fn actor_is_authenticated_returns_false_for_guest() {
    let actor = Actor {
        user_id: None,
        role: Role::Guest,
    };
    assert!(!actor.is_authenticated());
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Build a `JsonWebToken` service backed by the development RSA key
/// pair shipped at `share/default/jwt/`.
async fn jwt_service(cfg: Arc<Configuration>) -> JsonWebToken {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    {
        let mut settings = cfg.settings.write().await;
        settings.auth.private_key_path = Some(format!("{manifest_dir}/share/default/jwt/private.pem"));
        settings.auth.public_key_path = Some(format!("{manifest_dir}/share/default/jwt/public.pem"));
    }
    JsonWebToken::new(cfg).await
}

/// Build a complete `AppData` backed by an ephemeral `SQLite` database.
///
/// Only the `auth`, `user_repository`, `permissions`, and
/// `json_web_token` fields are exercised by the extractor; the rest
/// are constructed with defaults to satisfy the `AppData` struct.
#[allow(clippy::too_many_lines)]
async fn test_app_data(db_path: &str) -> Arc<AppData> {
    let cfg = Arc::new(Configuration::for_tests());

    // Point config at the temp database.
    {
        let mut settings = cfg.settings.write().await;
        settings.database.connect_url = url::Url::parse(&format!("sqlite://{db_path}?mode=rwc")).unwrap();
    }

    let db: Arc<Box<dyn database::Database>> =
        Arc::new(database::connect(&format!("sqlite://{db_path}?mode=rwc")).await.unwrap());

    let jwt = Arc::new(jwt_service(cfg.clone()).await);
    let auth = Arc::new(Authentication::new(jwt.clone(), db.clone()));

    let category_repo = Arc::new(DbCategoryRepository::new(db.clone()));
    let tag_repo = Arc::new(DbTagRepository::new(db.clone()));
    let user_repo: Arc<Box<dyn Repository>> = Arc::new(Box::new(DbUserRepository::new(db.clone())));
    let user_auth_repo = Arc::new(DbUserAuthenticationRepository::new(db.clone()));
    let user_profile_repo = Arc::new(DbUserProfileRepository::new(db.clone()));
    let torrent_repo = Arc::new(DbTorrentRepository::new(db.clone()));
    let canonical_info_hash_repo = Arc::new(DbCanonicalInfoHashGroupRepository::new(db.clone()));
    let torrent_info_repo = Arc::new(DbTorrentInfoRepository::new(db.clone()));
    let torrent_file_repo = Arc::new(DbTorrentFileRepository::new(db.clone()));
    let torrent_announce_url_repo = Arc::new(DbTorrentAnnounceUrlRepository::new(db.clone()));
    let torrent_tag_repo = Arc::new(DbTorrentTagRepository::new(db.clone()));
    let torrent_listing_gen = Arc::new(DbTorrentListingGenerator::new(db.clone()));
    let banned_user_list = Arc::new(DbBannedUserList::new(db.clone()));

    let permissions: Arc<dyn Permissions> = Arc::new(PermissionMatrix::default_matrix());

    let tracker_service = Arc::new(tracker::service::Service::new(cfg.clone(), db.clone()).await);
    let tracker_stats_importer = Arc::new(StatisticsImporter::new(cfg.clone(), tracker_service.clone(), db.clone()).await);
    let mailer_service = Arc::new(mailer::Service::new(cfg.clone(), jwt.clone()).await);
    let image_cache = Arc::new(ImageCacheService::new(cfg.clone()).await);

    let category_service = Arc::new(category::Service::new(category_repo.clone()));
    let tag_service = Arc::new(tag::Service::new(tag_repo.clone()));
    let proxy_service = Arc::new(proxy::Service::new(image_cache.clone()));
    let settings_service = Arc::new(settings::Service::new(cfg.clone()));
    let torrent_index = Arc::new(torrent::Index::new(
        cfg.clone(),
        tracker_stats_importer.clone(),
        tracker_service.clone(),
        category_repo.clone(),
        torrent_repo.clone(),
        canonical_info_hash_repo.clone(),
        torrent_info_repo.clone(),
        torrent_file_repo.clone(),
        torrent_announce_url_repo.clone(),
        torrent_tag_repo.clone(),
        torrent_listing_gen.clone(),
    ));
    let registration_service = Arc::new(user::RegistrationService::new(
        cfg.clone(),
        jwt.clone(),
        mailer_service.clone(),
        user_repo.clone(),
        user_profile_repo.clone(),
    ));
    let profile_service = Arc::new(user::ProfileService::new(cfg.clone(), user_auth_repo.clone()));
    let ban_service = Arc::new(user::BanService::new(user_profile_repo.clone(), banned_user_list.clone()));
    let authentication_service = Arc::new(AuthenticationService::new(
        cfg.clone(),
        jwt.clone(),
        db.clone(),
        user_repo.clone(),
        user_profile_repo.clone(),
        user_auth_repo.clone(),
    ));
    let about_service = Arc::new(about::Service::new());
    let listing_service = Arc::new(user::ListingService::new(cfg.clone(), user_profile_repo.clone()));

    Arc::new(AppData::new(
        cfg,
        db,
        jwt,
        auth,
        authentication_service,
        tracker_service,
        tracker_stats_importer,
        mailer_service,
        image_cache,
        permissions,
        category_repo,
        tag_repo,
        user_repo,
        user_auth_repo,
        user_profile_repo,
        torrent_repo,
        canonical_info_hash_repo,
        torrent_info_repo,
        torrent_file_repo,
        torrent_announce_url_repo,
        torrent_tag_repo,
        torrent_listing_gen,
        banned_user_list,
        category_service,
        tag_service,
        proxy_service,
        settings_service,
        torrent_index,
        registration_service,
        profile_service,
        ban_service,
        about_service,
        listing_service,
    ))
}

/// Register a user in the database and return (`user_id`, JWT).
async fn register_and_sign(app_data: &AppData, username: &str, role: &str) -> (UserId, String) {
    let user_id = app_data
        .database
        .insert_user_and_get_id(username, &format!("{username}@test.local"), "hashed_pw")
        .await
        .unwrap();

    if role == "admin" {
        app_data.database.grant_admin_role(user_id).await.unwrap();
    }

    let compact = app_data.user_repository.get_compact(&user_id).await.unwrap();
    let token_gen = app_data.database.get_token_generation(user_id).await.unwrap();
    let token = app_data.json_web_token.sign(compact, token_gen).await.unwrap();

    (user_id, token)
}

/// Build a minimal Axum router with two test handlers:
///
/// - `GET /guest-allowed` → guarded by `RequirePermission<GetCategories>` (guest-allowed)
/// - `GET /admin-only`    → guarded by `RequirePermission<AddCategory>` (admin-only)
fn test_router(app_data: Arc<AppData>) -> Router {
    async fn guest_allowed_handler(RequirePermission(_actor, _): RequirePermission<GetCategories>) -> Response {
        StatusCode::OK.into_response()
    }

    async fn admin_only_handler(RequirePermission(_actor, _): RequirePermission<AddCategory>) -> Response {
        StatusCode::OK.into_response()
    }

    Router::new()
        .route("/guest-allowed", get(guest_allowed_handler))
        .route("/admin-only", get(admin_only_handler))
        .with_state(app_data)
}

/// Send a GET request to the router.
async fn send_get(router: &Router, uri: &str, authorization: Option<&str>) -> hyper::Response<axum::body::Body> {
    let mut builder = hyper::Request::builder().uri(uri).method("GET");
    if let Some(auth) = authorization {
        builder = builder.header("Authorization", auth);
    }
    let req = builder.body(axum::body::Body::empty()).unwrap();
    router.clone().oneshot(req).await.unwrap()
}

// ── RequirePermission extractor (full path) ──────────────────────────

#[tokio::test]
async fn guest_denied_action_returns_401() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let app_data = test_app_data(tmp.path().to_str().unwrap()).await;
    let router = test_router(app_data);

    // No Authorization header → Guest role.
    // AddCategory is denied for Guest → 401.
    let resp = send_get(&router, "/admin-only", None).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn guest_allowed_action_returns_200() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let app_data = test_app_data(tmp.path().to_str().unwrap()).await;
    let router = test_router(app_data);

    // No Authorization header → Guest role.
    // GetCategories is allowed for Guest → 200.
    let resp = send_get(&router, "/guest-allowed", None).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn authenticated_denied_action_returns_403() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let app_data = test_app_data(tmp.path().to_str().unwrap()).await;

    let (_user_id, token) = register_and_sign(&app_data, "regularuser", "registered").await;
    let router = test_router(app_data);

    // Valid token, Registered role → AddCategory denied → 403.
    let resp = send_get(&router, "/admin-only", Some(&format!("Bearer {token}"))).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn authenticated_allowed_action_returns_200() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let app_data = test_app_data(tmp.path().to_str().unwrap()).await;

    let (_user_id, token) = register_and_sign(&app_data, "regularuser2", "registered").await;
    let router = test_router(app_data);

    // Valid token, Registered role → GetCategories allowed → 200.
    let resp = send_get(&router, "/guest-allowed", Some(&format!("Bearer {token}"))).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn admin_allowed_for_all_actions() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let app_data = test_app_data(tmp.path().to_str().unwrap()).await;

    let (_user_id, token) = register_and_sign(&app_data, "adminuser", "admin").await;
    let router = test_router(app_data);

    // Admin role → AddCategory allowed → 200.
    let resp = send_get(&router, "/admin-only", Some(&format!("Bearer {token}"))).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
