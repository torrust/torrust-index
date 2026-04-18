use std::borrow::Cow;

use hyper::StatusCode;
use thiserror::Error;
use tracing::error;

use crate::databases::database;
use crate::models::torrent::MetadataError;
use crate::tracker::service::TrackerAPIError;
use crate::utils::parse_torrent::DecodeTorrentFileError;

// ── AuthError ────────────────────────────────────────────────────────

/// Domain error for authentication and authorization.
///
/// Covers JWT verification, password verification, and role-based
/// permission checks.  Status-code mapping is co-located via
/// [`AuthError::status_code`].
#[derive(Debug, PartialEq, Eq, Error)]
pub enum AuthError {
    #[error("Invalid username/email or password")]
    WrongPasswordOrUsername,

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Username not found")]
    UsernameNotFound,

    #[error("Token not found. Please sign in.")]
    TokenNotFound,

    #[error("Token expired. Please sign in again.")]
    TokenExpired,

    #[error("Token invalid.")]
    TokenInvalid,

    #[error("Token has been revoked. Please sign in again.")]
    TokenRevoked,

    #[error("Unauthorized action.")]
    UnauthorizedAction,

    #[error("Unauthorized actions for guest users. Try logging in to check if you have permission to perform the action")]
    UnauthorizedActionForGuests,

    #[error("Authentication error, please sign in")]
    LoggedInUserNotFound,

    #[error("Please verify your email before logging in")]
    EmailNotVerified,

    #[error("internal server error")]
    InternalServerError,

    #[error("Database error.")]
    DatabaseError,

    #[error("User not found")]
    UserNotFound,

    #[error("Unrecognised user role. Contact admin.")]
    UnrecognisedRole,
}

impl AuthError {
    /// HTTP status code for this error variant.
    #[must_use]
    pub const fn status_code(&self) -> StatusCode {
        match self {
            Self::WrongPasswordOrUsername | Self::InvalidPassword | Self::EmailNotVerified | Self::UnauthorizedAction => {
                StatusCode::FORBIDDEN
            }
            Self::UsernameNotFound | Self::UserNotFound => StatusCode::NOT_FOUND,
            Self::TokenNotFound
            | Self::TokenExpired
            | Self::TokenInvalid
            | Self::TokenRevoked
            | Self::LoggedInUserNotFound
            | Self::UnauthorizedActionForGuests => StatusCode::UNAUTHORIZED,
            Self::InternalServerError | Self::DatabaseError | Self::UnrecognisedRole => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<database::Error> for AuthError {
    fn from(e: database::Error) -> Self {
        if matches!(e, database::Error::UserNotFound) {
            Self::UserNotFound
        } else {
            error!(error = %e, "database error (auth)");
            Self::DatabaseError
        }
    }
}

impl From<argon2::password_hash::Error> for AuthError {
    fn from(e: argon2::password_hash::Error) -> Self {
        error!(error = %e, "password hashing error");
        Self::InternalServerError
    }
}

// ── UserError ────────────────────────────────────────────────────────

/// Domain error for user management operations.
///
/// Covers registration, password changes, banning, and user listings.
/// Status-code mapping is co-located via [`UserError::status_code`].
#[derive(Debug, PartialEq, Eq, Error)]
pub enum UserError {
    #[error("This server is closed for registration. Contact admin if this is unexpected")]
    ClosedForRegistration,

    #[error("Email is required")]
    EmailMissing,

    #[error("Please enter a valid email address")]
    EmailInvalid,

    #[error("Email not available")]
    EmailTaken,

    #[error("Please verify your email before logging in")]
    EmailNotVerified,

    #[error("Failed to send verification email.")]
    FailedToSendVerificationEmail,

    #[error("User not found")]
    UserNotFound,

    #[error("Account not found")]
    AccountNotFound,

    #[error("Username not available")]
    UsernameTaken,

    #[error("Invalid username. Usernames must consist of 1-20 alphanumeric characters, dashes, or underscore")]
    UsernameInvalid,

    #[error("Can't allow profanity in usernames")]
    ProfanityError,

    #[error("Username contains blacklisted words")]
    BlacklistError,

    #[error("username_case_mapped violation")]
    UsernameCaseMappedError,

    #[error("Password too short")]
    PasswordTooShort,

    #[error("Password too long")]
    PasswordTooLong,

    #[error("Passwords don't match")]
    PasswordsDontMatch,

    #[error("Invalid user listing fields in the URL params.")]
    InvalidUserListing,

    #[error("Invalid password")]
    InvalidPassword,

    // -- Cross-cutting (temporary, see ADR-T-006) --
    #[error("Unauthorized action.")]
    UnauthorizedAction,

    #[error("Unauthorized actions for guest users. Try logging in to check if you have permission to perform the action")]
    UnauthorizedActionForGuests,

    #[error("Database error.")]
    DatabaseError,

    #[error("internal server error")]
    InternalServerError,
}

impl UserError {
    /// HTTP status code for this error variant.
    #[must_use]
    pub const fn status_code(&self) -> StatusCode {
        match self {
            Self::ClosedForRegistration | Self::EmailNotVerified | Self::UnauthorizedAction | Self::InvalidPassword => {
                StatusCode::FORBIDDEN
            }
            Self::EmailMissing | Self::UserNotFound | Self::AccountNotFound => StatusCode::NOT_FOUND,
            Self::EmailInvalid
            | Self::EmailTaken
            | Self::UsernameTaken
            | Self::UsernameInvalid
            | Self::ProfanityError
            | Self::BlacklistError
            | Self::UsernameCaseMappedError
            | Self::PasswordTooShort
            | Self::PasswordTooLong
            | Self::PasswordsDontMatch
            | Self::InvalidUserListing => StatusCode::BAD_REQUEST,
            Self::UnauthorizedActionForGuests => StatusCode::UNAUTHORIZED,
            Self::FailedToSendVerificationEmail | Self::DatabaseError | Self::InternalServerError => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

impl From<AuthError> for UserError {
    fn from(e: AuthError) -> Self {
        match e {
            AuthError::UnauthorizedAction => Self::UnauthorizedAction,
            AuthError::UnauthorizedActionForGuests => Self::UnauthorizedActionForGuests,
            AuthError::InvalidPassword => Self::InvalidPassword,
            _ => Self::InternalServerError,
        }
    }
}

impl From<database::Error> for UserError {
    fn from(e: database::Error) -> Self {
        match e {
            database::Error::UsernameTaken => Self::UsernameTaken,
            database::Error::EmailTaken => Self::EmailTaken,
            database::Error::UserNotFound => Self::UserNotFound,
            _ => {
                error!(error = %e, "database error (user)");
                Self::DatabaseError
            }
        }
    }
}

impl From<argon2::password_hash::Error> for UserError {
    fn from(e: argon2::password_hash::Error) -> Self {
        error!(error = %e, "password hashing error");
        Self::InternalServerError
    }
}

impl From<serde_json::Error> for UserError {
    fn from(e: serde_json::Error) -> Self {
        error!(error = %e, "JSON error");
        Self::InternalServerError
    }
}

// ── TorrentError ─────────────────────────────────────────────────────

/// Domain error for torrent operations.
///
/// Covers upload, download, listing, and update of torrents.
/// Status-code mapping is co-located via [`TorrentError::status_code`].
#[derive(Debug, PartialEq, Eq, Error)]
pub enum TorrentError {
    #[error("Uploaded torrent is not valid.")]
    InvalidTorrentFile,

    #[error("Uploaded torrent has an invalid pieces key.")]
    InvalidTorrentPiecesLength,

    #[error("Only .torrent files can be uploaded.")]
    InvalidFileType,

    #[error("Torrent title is too short.")]
    InvalidTorrentTitleLength,

    #[error("Some mandatory metadata fields are missing.")]
    MissingMandatoryMetadataFields,

    #[error("Selected category does not exist.")]
    InvalidCategory,

    #[error("Selected tag does not exist.")]
    InvalidTag,

    #[error("This torrent already exists in our database.")]
    InfoHashAlreadyExists,

    #[error("A torrent with the same canonical infohash already exists in our database.")]
    CanonicalInfoHashAlreadyExists,

    #[error("A torrent with the same original infohash already exists in our database.")]
    OriginalInfoHashAlreadyExists,

    #[error("This torrent title has already been used.")]
    TorrentTitleAlreadyExists,

    #[error("Could not whitelist torrent.")]
    WhitelistingError,

    #[error("Torrent not found.")]
    TorrentNotFound,

    // -- Tracker errors embedded here --
    #[error("Sorry, we have an error with our tracker connection.")]
    TrackerOffline,

    #[error("Tracker response error. The operation could not be performed.")]
    TrackerResponseError,

    #[error("Tracker unknown response. Unexpected response from tracker. For example, if it can't be parsed.")]
    TrackerUnknownResponse,

    #[error("Torrent not found in tracker.")]
    TorrentNotFoundInTracker,

    #[error("Invalid tracker API token.")]
    InvalidTrackerToken,

    // -- Cross-cutting (temporary, see ADR-T-006) --
    #[error("Unauthorized action.")]
    UnauthorizedAction,

    #[error("Unauthorized actions for guest users. Try logging in to check if you have permission to perform the action")]
    UnauthorizedActionForGuests,

    #[error("Database error.")]
    DatabaseError,

    #[error("internal server error")]
    InternalServerError,
}

impl TorrentError {
    /// HTTP status code for this error variant.
    #[must_use]
    pub const fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidTorrentFile
            | Self::InvalidTorrentPiecesLength
            | Self::InvalidFileType
            | Self::InvalidTorrentTitleLength
            | Self::MissingMandatoryMetadataFields
            | Self::InvalidCategory
            | Self::InvalidTag
            | Self::InfoHashAlreadyExists
            | Self::TorrentTitleAlreadyExists => StatusCode::BAD_REQUEST,
            Self::CanonicalInfoHashAlreadyExists | Self::OriginalInfoHashAlreadyExists => StatusCode::CONFLICT,
            Self::TorrentNotFound | Self::TorrentNotFoundInTracker => StatusCode::NOT_FOUND,
            Self::TrackerOffline => StatusCode::SERVICE_UNAVAILABLE,
            Self::UnauthorizedAction => StatusCode::FORBIDDEN,
            Self::UnauthorizedActionForGuests => StatusCode::UNAUTHORIZED,
            Self::WhitelistingError
            | Self::TrackerResponseError
            | Self::TrackerUnknownResponse
            | Self::InvalidTrackerToken
            | Self::DatabaseError
            | Self::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<AuthError> for TorrentError {
    fn from(e: AuthError) -> Self {
        match e {
            AuthError::UnauthorizedAction => Self::UnauthorizedAction,
            AuthError::UnauthorizedActionForGuests => Self::UnauthorizedActionForGuests,
            _ => Self::InternalServerError,
        }
    }
}

impl From<database::Error> for TorrentError {
    fn from(e: database::Error) -> Self {
        match e {
            database::Error::TorrentNotFound | database::Error::TorrentInfoHashNotFound => Self::TorrentNotFound,
            database::Error::TorrentAlreadyExists => Self::InfoHashAlreadyExists,
            database::Error::TorrentTitleAlreadyExists => Self::TorrentTitleAlreadyExists,
            database::Error::CategoryNotFound => Self::InvalidCategory,
            database::Error::TagNotFound => Self::InvalidTag,
            _ => {
                error!(error = %e, "database error (torrent)");
                Self::DatabaseError
            }
        }
    }
}

impl From<sqlx::Error> for TorrentError {
    fn from(e: sqlx::Error) -> Self {
        error!(error = %e, "sqlx error");

        if let Some(err) = e.as_database_error() {
            return if err.code() == Some(Cow::from("2067")) {
                if err.message().contains("torrust_torrents.info_hash") {
                    error!("info_hash already exists: {}", err.message());
                    Self::InfoHashAlreadyExists
                } else {
                    Self::InternalServerError
                }
            } else {
                Self::DatabaseError
            };
        }

        if matches!(e, sqlx::Error::RowNotFound) {
            return Self::TorrentNotFound;
        }

        Self::InternalServerError
    }
}

impl From<MetadataError> for TorrentError {
    fn from(e: MetadataError) -> Self {
        error!(error = %e, "metadata error");
        match e {
            MetadataError::MissingTorrentTitle => Self::MissingMandatoryMetadataFields,
            MetadataError::InvalidTorrentTitleLength => Self::InvalidTorrentTitleLength,
        }
    }
}

impl From<DecodeTorrentFileError> for TorrentError {
    fn from(e: DecodeTorrentFileError) -> Self {
        error!(error = %e, "torrent file decode error");
        match e {
            DecodeTorrentFileError::InvalidTorrentPiecesLength => Self::InvalidTorrentPiecesLength,
            DecodeTorrentFileError::CannotBencodeInfoDict
            | DecodeTorrentFileError::InvalidInfoDictionary
            | DecodeTorrentFileError::InvalidBencodeData => Self::InvalidTorrentFile,
        }
    }
}

impl From<TrackerAPIError> for TorrentError {
    fn from(e: TrackerAPIError) -> Self {
        error!(error = %e, "tracker API error");
        match e {
            TrackerAPIError::TrackerOffline { error: _ } => Self::TrackerOffline,
            TrackerAPIError::InternalServerError | TrackerAPIError::NotFound => Self::TrackerResponseError,
            TrackerAPIError::TorrentNotFound => Self::TorrentNotFoundInTracker,
            TrackerAPIError::UnexpectedResponseStatus
            | TrackerAPIError::MissingResponseBody
            | TrackerAPIError::FailedToParseTrackerResponse { body: _ } => Self::TrackerUnknownResponse,
            TrackerAPIError::CannotSaveUserKey => Self::DatabaseError,
            TrackerAPIError::InvalidToken => Self::InvalidTrackerToken,
        }
    }
}

impl From<std::io::Error> for TorrentError {
    fn from(e: std::io::Error) -> Self {
        error!(error = %e, "I/O error");
        Self::InternalServerError
    }
}

impl From<serde_json::Error> for TorrentError {
    fn from(e: serde_json::Error) -> Self {
        error!(error = %e, "JSON error");
        Self::InternalServerError
    }
}

impl From<Box<dyn std::error::Error>> for TorrentError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        error!(error = %e, "boxed error");
        Self::InternalServerError
    }
}

// ── CategoryTagError ─────────────────────────────────────────────────

/// Domain error for category and tag operations.
///
/// Covers all failure modes from the category and tag services.
/// Status-code mapping is co-located via [`CategoryTagError::status_code`].
#[derive(Debug, PartialEq, Eq, Error)]
pub enum CategoryTagError {
    #[error("Selected category does not exist.")]
    InvalidCategory,

    #[error("Category already exists.")]
    CategoryAlreadyExists,

    #[error("Category name cannot be empty.")]
    CategoryNameEmpty,

    #[error("Category not found.")]
    CategoryNotFound,

    #[error("Selected tag does not exist.")]
    InvalidTag,

    #[error("Tag already exists.")]
    TagAlreadyExists,

    #[error("Tag name cannot be empty.")]
    TagNameEmpty,

    #[error("Tag not found.")]
    TagNotFound,

    // -- Cross-cutting (temporary, see ADR-T-006) --
    #[error("Unauthorized action.")]
    UnauthorizedAction,

    #[error("Unauthorized actions for guest users. Try logging in to check if you have permission to perform the action")]
    UnauthorizedActionForGuests,

    #[error("Database error.")]
    DatabaseError,
}

impl CategoryTagError {
    /// HTTP status code for this error variant.
    #[must_use]
    pub const fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidCategory
            | Self::CategoryAlreadyExists
            | Self::CategoryNameEmpty
            | Self::InvalidTag
            | Self::TagAlreadyExists
            | Self::TagNameEmpty => StatusCode::BAD_REQUEST,
            Self::CategoryNotFound | Self::TagNotFound => StatusCode::NOT_FOUND,
            Self::UnauthorizedAction => StatusCode::FORBIDDEN,
            Self::UnauthorizedActionForGuests => StatusCode::UNAUTHORIZED,
            Self::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<AuthError> for CategoryTagError {
    fn from(e: AuthError) -> Self {
        match e {
            AuthError::UnauthorizedAction => Self::UnauthorizedAction,
            AuthError::UnauthorizedActionForGuests => Self::UnauthorizedActionForGuests,
            _ => Self::DatabaseError,
        }
    }
}

// ── ApiError ─────────────────────────────────────────────────────────

/// Thin wrapper that unifies all domain errors at the HTTP boundary.
///
/// Handlers that call multiple services returning different domain error
/// types can use `ApiError` as their error type, relying on `From` impls
/// to convert each domain error automatically.
///
/// `ApiError` implements [`axum::response::IntoResponse`] by delegating to
/// the wrapped domain error's `status_code()` and `Display` message.
#[derive(Debug, Error)]
pub enum ApiError {
    #[error(transparent)]
    Auth(#[from] AuthError),

    #[error(transparent)]
    User(#[from] UserError),

    #[error(transparent)]
    Torrent(#[from] TorrentError),

    #[error(transparent)]
    CategoryTag(#[from] CategoryTagError),
}

impl ApiError {
    /// HTTP status code for this error, delegated to the inner domain error.
    #[must_use]
    pub const fn status_code(&self) -> StatusCode {
        match self {
            Self::Auth(e) => e.status_code(),
            Self::User(e) => e.status_code(),
            Self::Torrent(e) => e.status_code(),
            Self::CategoryTag(e) => e.status_code(),
        }
    }
}
