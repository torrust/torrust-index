use std::borrow::Cow;
use std::error;

use hyper::StatusCode;
use thiserror::Error;
use tracing::error;

use crate::databases::database;
use crate::models::torrent::MetadataError;
use crate::tracker::service::TrackerAPIError;
use crate::utils::parse_torrent::DecodeTorrentFileError;

pub type ServiceResult<V> = Result<V, ServiceError>;

/// Domain error for category and tag operations.
///
/// Covers all failure modes from the category and tag services.
/// Status-code mapping is co-located via [`CategoryTagError::status_code`].
///
/// The `UnauthorizedAction`, `UnauthorizedActionForGuests`, and
/// `DatabaseError` variants are cross-cutting placeholders that will
/// be replaced by dedicated `AuthError` / `InfraError` enums in a
/// later phase (see ADR-T-006 Phase 1).
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

/// Transitional conversion: the authorization service still returns
/// [`ServiceError`]; this maps the auth-related variants into
/// [`CategoryTagError`].  Will be removed once `AuthError` is
/// extracted (ADR-T-006 Phase 1, auth domain).
impl From<ServiceError> for CategoryTagError {
    fn from(e: ServiceError) -> Self {
        match e {
            ServiceError::UnauthorizedAction => Self::UnauthorizedAction,
            ServiceError::UnauthorizedActionForGuests => Self::UnauthorizedActionForGuests,
            _ => Self::DatabaseError,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Error)]
#[allow(dead_code)]
pub enum ServiceError {
    #[error("internal server error")]
    InternalServerError,

    #[error("This server is is closed for registration. Contact admin if this is unexpected")]
    ClosedForRegistration,

    #[error("Email is required")] //405j
    EmailMissing,
    #[error("Please enter a valid email address")] //405j
    EmailInvalid,

    #[error("The value you entered for URL is not a URL")] //405j
    NotAUrl,

    #[error("Invalid username/email or password")]
    WrongPasswordOrUsername,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Username not found")]
    UsernameNotFound,
    #[error("User not found")]
    UserNotFound,

    #[error("Account not found")]
    AccountNotFound,

    /// when the value passed contains profanity
    #[error("Can't allow profanity in usernames")]
    ProfanityError,
    /// when the value passed contains blacklisted words
    /// see [blacklist](https://github.com/shuttlecraft/The-Big-Username-Blacklist)
    #[error("Username contains blacklisted words")]
    BlacklistError,
    /// when the value passed contains characters not present
    /// in [UsernameCaseMapped](https://tools.ietf.org/html/rfc8265#page-7)
    /// profile
    #[error("username_case_mapped violation")]
    UsernameCaseMappedError,

    #[error("Password too short")]
    PasswordTooShort,
    #[error("Password too long")]
    PasswordTooLong,
    #[error("Passwords don't match")]
    PasswordsDontMatch,

    /// when the a username is already taken
    #[error("Username not available")]
    UsernameTaken,

    #[error("Invalid username. Usernames must consist of 1-20 alphanumeric characters, dashes, or underscore")]
    UsernameInvalid,

    /// email is already taken
    #[error("Email not available")]
    EmailTaken,

    #[error("Please verify your email before logging in")]
    EmailNotVerified,

    /// when the a token name is already taken
    /// token not found
    #[error("Token not found. Please sign in.")]
    TokenNotFound,

    /// token expired
    #[error("Token expired. Please sign in again.")]
    TokenExpired,

    /// token invalid
    #[error("Token invalid.")]
    TokenInvalid,

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

    #[error("Unauthorized action.")]
    UnauthorizedAction,

    #[error("Unauthorized actions for guest users. Try logging in to check if you have permission to perform the action")]
    UnauthorizedActionForGuests,

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

    #[error("Failed to send verification email.")]
    FailedToSendVerificationEmail,

    #[error("Torrent not found.")]
    TorrentNotFound,

    #[error("Database error.")]
    DatabaseError,

    #[error("Authentication error, please sign in")]
    LoggedInUserNotFound,

    // Begin tracker errors
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
    // End tracker errors
    #[error("Invalid user listing fields in the URL params.")]
    InvalidUserListing,
}

impl From<sqlx::Error> for ServiceError {
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
                Self::TorrentNotFound
            };
        }

        Self::InternalServerError
    }
}

impl From<database::Error> for ServiceError {
    fn from(e: database::Error) -> Self {
        map_database_error_to_service_error(&e)
    }
}

impl From<argon2::password_hash::Error> for ServiceError {
    fn from(e: argon2::password_hash::Error) -> Self {
        error!(error = %e, "password hashing error");
        Self::InternalServerError
    }
}

impl From<std::io::Error> for ServiceError {
    fn from(e: std::io::Error) -> Self {
        error!(error = %e, "I/O error");
        Self::InternalServerError
    }
}

impl From<Box<dyn error::Error>> for ServiceError {
    fn from(e: Box<dyn error::Error>) -> Self {
        error!(error = %e, "boxed error");
        Self::InternalServerError
    }
}

impl From<serde_json::Error> for ServiceError {
    fn from(e: serde_json::Error) -> Self {
        error!(error = %e, "JSON error");
        Self::InternalServerError
    }
}

impl From<MetadataError> for ServiceError {
    fn from(e: MetadataError) -> Self {
        error!(error = %e, "metadata error");
        match e {
            MetadataError::MissingTorrentTitle => Self::MissingMandatoryMetadataFields,
            MetadataError::InvalidTorrentTitleLength => Self::InvalidTorrentTitleLength,
        }
    }
}

impl From<DecodeTorrentFileError> for ServiceError {
    fn from(e: DecodeTorrentFileError) -> Self {
        error!(error = %e, "torrent file decode error");
        match e {
            DecodeTorrentFileError::InvalidTorrentPiecesLength => Self::InvalidTorrentTitleLength,
            DecodeTorrentFileError::CannotBencodeInfoDict
            | DecodeTorrentFileError::InvalidInfoDictionary
            | DecodeTorrentFileError::InvalidBencodeData => Self::InvalidTorrentFile,
        }
    }
}

impl From<TrackerAPIError> for ServiceError {
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

#[must_use]
pub const fn http_status_code_for_service_error(error: &ServiceError) -> StatusCode {
    #[allow(clippy::match_same_arms)]
    match error {
        ServiceError::ClosedForRegistration => StatusCode::FORBIDDEN,
        ServiceError::EmailInvalid => StatusCode::BAD_REQUEST,
        ServiceError::NotAUrl => StatusCode::BAD_REQUEST,
        ServiceError::WrongPasswordOrUsername => StatusCode::FORBIDDEN,
        ServiceError::InvalidPassword => StatusCode::FORBIDDEN,
        ServiceError::UsernameNotFound => StatusCode::NOT_FOUND,
        ServiceError::UserNotFound => StatusCode::NOT_FOUND,
        ServiceError::AccountNotFound => StatusCode::NOT_FOUND,
        ServiceError::ProfanityError => StatusCode::BAD_REQUEST,
        ServiceError::BlacklistError => StatusCode::BAD_REQUEST,
        ServiceError::UsernameCaseMappedError => StatusCode::BAD_REQUEST,
        ServiceError::PasswordTooShort => StatusCode::BAD_REQUEST,
        ServiceError::PasswordTooLong => StatusCode::BAD_REQUEST,
        ServiceError::PasswordsDontMatch => StatusCode::BAD_REQUEST,
        ServiceError::UsernameTaken => StatusCode::BAD_REQUEST,
        ServiceError::UsernameInvalid => StatusCode::BAD_REQUEST,
        ServiceError::EmailTaken => StatusCode::BAD_REQUEST,
        ServiceError::EmailNotVerified => StatusCode::FORBIDDEN,
        ServiceError::TokenNotFound => StatusCode::UNAUTHORIZED,
        ServiceError::TokenExpired => StatusCode::UNAUTHORIZED,
        ServiceError::TokenInvalid => StatusCode::UNAUTHORIZED,
        ServiceError::TorrentNotFound => StatusCode::NOT_FOUND,
        ServiceError::InvalidTorrentFile => StatusCode::BAD_REQUEST,
        ServiceError::InvalidTorrentPiecesLength => StatusCode::BAD_REQUEST,
        ServiceError::InvalidFileType => StatusCode::BAD_REQUEST,
        ServiceError::InvalidTorrentTitleLength => StatusCode::BAD_REQUEST,
        ServiceError::MissingMandatoryMetadataFields => StatusCode::BAD_REQUEST,
        ServiceError::InvalidCategory => StatusCode::BAD_REQUEST,
        ServiceError::InvalidTag => StatusCode::BAD_REQUEST,
        ServiceError::UnauthorizedAction => StatusCode::FORBIDDEN,
        ServiceError::UnauthorizedActionForGuests => StatusCode::UNAUTHORIZED,
        ServiceError::InfoHashAlreadyExists => StatusCode::BAD_REQUEST,
        ServiceError::CanonicalInfoHashAlreadyExists => StatusCode::CONFLICT,
        ServiceError::OriginalInfoHashAlreadyExists => StatusCode::CONFLICT,
        ServiceError::TorrentTitleAlreadyExists => StatusCode::BAD_REQUEST,
        ServiceError::TrackerOffline => StatusCode::SERVICE_UNAVAILABLE,
        ServiceError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        ServiceError::EmailMissing => StatusCode::NOT_FOUND,
        ServiceError::FailedToSendVerificationEmail => StatusCode::INTERNAL_SERVER_ERROR,
        ServiceError::WhitelistingError => StatusCode::INTERNAL_SERVER_ERROR,
        ServiceError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        ServiceError::TrackerResponseError => StatusCode::INTERNAL_SERVER_ERROR,
        ServiceError::TrackerUnknownResponse => StatusCode::INTERNAL_SERVER_ERROR,
        ServiceError::TorrentNotFoundInTracker => StatusCode::NOT_FOUND,
        ServiceError::InvalidTrackerToken => StatusCode::INTERNAL_SERVER_ERROR,
        ServiceError::LoggedInUserNotFound => StatusCode::UNAUTHORIZED,
        ServiceError::InvalidUserListing => StatusCode::BAD_REQUEST,
    }
}

#[must_use]
pub const fn map_database_error_to_service_error(error: &database::Error) -> ServiceError {
    #[allow(clippy::match_same_arms)]
    match error {
        database::Error::Error => ServiceError::InternalServerError,
        database::Error::ErrorWithText(_) => ServiceError::InternalServerError,
        database::Error::UsernameTaken => ServiceError::UsernameTaken,
        database::Error::EmailTaken => ServiceError::EmailTaken,
        database::Error::UserNotFound => ServiceError::UserNotFound,
        database::Error::CategoryNotFound => ServiceError::InvalidCategory,
        database::Error::TagAlreadyExists => ServiceError::InternalServerError,
        database::Error::TagNotFound => ServiceError::InvalidTag,
        database::Error::TorrentNotFound => ServiceError::TorrentNotFound,
        database::Error::TorrentAlreadyExists => ServiceError::InfoHashAlreadyExists,
        database::Error::TorrentTitleAlreadyExists => ServiceError::TorrentTitleAlreadyExists,
        database::Error::UnrecognizedDatabaseDriver => ServiceError::InternalServerError,
        database::Error::TorrentInfoHashNotFound => ServiceError::TorrentNotFound,
    }
}
