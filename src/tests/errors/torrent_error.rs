//! `TorrentError` status-code, display, and `From` impl tests (§T-006 §1–3).

use std::io;

use hyper::StatusCode;

use crate::databases::database;
use crate::errors::{AuthError, TorrentError};
use crate::models::torrent::MetadataError;
use crate::tracker::service::TrackerAPIError;
use crate::utils::parse_torrent::DecodeTorrentFileError;

// ── §1 Status-code mapping ──────────────────────────────────────────

#[test]
fn invalid_torrent_file_returns_bad_request() {
    assert_eq!(TorrentError::InvalidTorrentFile.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn invalid_torrent_pieces_length_returns_bad_request() {
    assert_eq!(
        TorrentError::InvalidTorrentPiecesLength.status_code(),
        StatusCode::BAD_REQUEST
    );
}

#[test]
fn invalid_file_type_returns_bad_request() {
    assert_eq!(TorrentError::InvalidFileType.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn invalid_torrent_title_length_returns_bad_request() {
    assert_eq!(TorrentError::InvalidTorrentTitleLength.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn missing_mandatory_metadata_fields_returns_bad_request() {
    assert_eq!(
        TorrentError::MissingMandatoryMetadataFields.status_code(),
        StatusCode::BAD_REQUEST
    );
}

#[test]
fn invalid_category_returns_bad_request() {
    assert_eq!(TorrentError::InvalidCategory.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn invalid_tag_returns_bad_request() {
    assert_eq!(TorrentError::InvalidTag.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn info_hash_already_exists_returns_bad_request() {
    assert_eq!(TorrentError::InfoHashAlreadyExists.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn canonical_info_hash_already_exists_returns_conflict() {
    assert_eq!(
        TorrentError::CanonicalInfoHashAlreadyExists.status_code(),
        StatusCode::CONFLICT
    );
}

#[test]
fn original_info_hash_already_exists_returns_conflict() {
    assert_eq!(
        TorrentError::OriginalInfoHashAlreadyExists.status_code(),
        StatusCode::CONFLICT
    );
}

#[test]
fn torrent_title_already_exists_returns_bad_request() {
    assert_eq!(TorrentError::TorrentTitleAlreadyExists.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn whitelisting_error_returns_500() {
    assert_eq!(
        TorrentError::WhitelistingError.status_code(),
        StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[test]
fn torrent_not_found_returns_not_found() {
    assert_eq!(TorrentError::TorrentNotFound.status_code(), StatusCode::NOT_FOUND);
}

#[test]
fn tracker_offline_returns_service_unavailable() {
    assert_eq!(TorrentError::TrackerOffline.status_code(), StatusCode::SERVICE_UNAVAILABLE);
}

#[test]
fn tracker_response_error_returns_500() {
    assert_eq!(
        TorrentError::TrackerResponseError.status_code(),
        StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[test]
fn tracker_unknown_response_returns_500() {
    assert_eq!(
        TorrentError::TrackerUnknownResponse.status_code(),
        StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[test]
fn torrent_not_found_in_tracker_returns_not_found() {
    assert_eq!(TorrentError::TorrentNotFoundInTracker.status_code(), StatusCode::NOT_FOUND);
}

#[test]
fn invalid_tracker_token_returns_500() {
    assert_eq!(
        TorrentError::InvalidTrackerToken.status_code(),
        StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[test]
fn unauthorized_action_returns_forbidden() {
    assert_eq!(TorrentError::UnauthorizedAction.status_code(), StatusCode::FORBIDDEN);
}

#[test]
fn unauthorized_action_for_guests_returns_unauthorized() {
    assert_eq!(
        TorrentError::UnauthorizedActionForGuests.status_code(),
        StatusCode::UNAUTHORIZED
    );
}

#[test]
fn database_error_returns_500() {
    assert_eq!(TorrentError::DatabaseError.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn internal_server_error_returns_500() {
    assert_eq!(
        TorrentError::InternalServerError.status_code(),
        StatusCode::INTERNAL_SERVER_ERROR
    );
}

// ── §2 Display messages ─────────────────────────────────────────────

#[test]
fn display_invalid_torrent_file() {
    assert_eq!(TorrentError::InvalidTorrentFile.to_string(), "Uploaded torrent is not valid.");
}

#[test]
fn display_invalid_torrent_pieces_length() {
    assert_eq!(
        TorrentError::InvalidTorrentPiecesLength.to_string(),
        "Uploaded torrent has an invalid pieces key."
    );
}

#[test]
fn display_invalid_file_type() {
    assert_eq!(
        TorrentError::InvalidFileType.to_string(),
        "Only .torrent files can be uploaded."
    );
}

#[test]
fn display_invalid_torrent_title_length() {
    assert_eq!(
        TorrentError::InvalidTorrentTitleLength.to_string(),
        "Torrent title is too short."
    );
}

#[test]
fn display_missing_mandatory_metadata_fields() {
    assert_eq!(
        TorrentError::MissingMandatoryMetadataFields.to_string(),
        "Some mandatory metadata fields are missing."
    );
}

#[test]
fn display_invalid_category() {
    assert_eq!(TorrentError::InvalidCategory.to_string(), "Selected category does not exist.");
}

#[test]
fn display_invalid_tag() {
    assert_eq!(TorrentError::InvalidTag.to_string(), "Selected tag does not exist.");
}

#[test]
fn display_info_hash_already_exists() {
    assert_eq!(
        TorrentError::InfoHashAlreadyExists.to_string(),
        "This torrent already exists in our database."
    );
}

#[test]
fn display_canonical_info_hash_already_exists() {
    assert_eq!(
        TorrentError::CanonicalInfoHashAlreadyExists.to_string(),
        "A torrent with the same canonical infohash already exists in our database."
    );
}

#[test]
fn display_original_info_hash_already_exists() {
    assert_eq!(
        TorrentError::OriginalInfoHashAlreadyExists.to_string(),
        "A torrent with the same original infohash already exists in our database."
    );
}

#[test]
fn display_torrent_title_already_exists() {
    assert_eq!(
        TorrentError::TorrentTitleAlreadyExists.to_string(),
        "This torrent title has already been used."
    );
}

#[test]
fn display_whitelisting_error() {
    assert_eq!(TorrentError::WhitelistingError.to_string(), "Could not whitelist torrent.");
}

#[test]
fn display_torrent_not_found() {
    assert_eq!(TorrentError::TorrentNotFound.to_string(), "Torrent not found.");
}

#[test]
fn display_tracker_offline() {
    assert_eq!(
        TorrentError::TrackerOffline.to_string(),
        "Sorry, we have an error with our tracker connection."
    );
}

#[test]
fn display_tracker_response_error() {
    assert_eq!(
        TorrentError::TrackerResponseError.to_string(),
        "Tracker response error. The operation could not be performed."
    );
}

#[test]
fn display_tracker_unknown_response() {
    assert_eq!(
        TorrentError::TrackerUnknownResponse.to_string(),
        "Tracker unknown response. Unexpected response from tracker. For example, if it can't be parsed."
    );
}

#[test]
fn display_torrent_not_found_in_tracker() {
    assert_eq!(
        TorrentError::TorrentNotFoundInTracker.to_string(),
        "Torrent not found in tracker."
    );
}

#[test]
fn display_invalid_tracker_token() {
    assert_eq!(TorrentError::InvalidTrackerToken.to_string(), "Invalid tracker API token.");
}

#[test]
fn display_unauthorized_action() {
    assert_eq!(TorrentError::UnauthorizedAction.to_string(), "Unauthorized action.");
}

#[test]
fn display_unauthorized_action_for_guests() {
    assert_eq!(
        TorrentError::UnauthorizedActionForGuests.to_string(),
        "Unauthorized actions for guest users. Try logging in to check if you have permission to perform the action"
    );
}

#[test]
fn display_database_error() {
    assert_eq!(TorrentError::DatabaseError.to_string(), "Database error.");
}

#[test]
fn display_internal_server_error() {
    assert_eq!(TorrentError::InternalServerError.to_string(), "internal server error");
}

// ── §3 From impl coverage ──────────────────────────────────────────

// From<AuthError>

#[test]
fn from_auth_unauthorized_action() {
    let err: TorrentError = AuthError::UnauthorizedAction.into();
    assert_eq!(err, TorrentError::UnauthorizedAction);
}

#[test]
fn from_auth_unauthorized_action_for_guests() {
    let err: TorrentError = AuthError::UnauthorizedActionForGuests.into();
    assert_eq!(err, TorrentError::UnauthorizedActionForGuests);
}

#[test]
fn from_auth_fallback() {
    let err: TorrentError = AuthError::TokenExpired.into();
    assert_eq!(err, TorrentError::InternalServerError);
}

// From<database::Error>

#[test]
fn from_database_torrent_not_found() {
    let err: TorrentError = database::Error::TorrentNotFound.into();
    assert_eq!(err, TorrentError::TorrentNotFound);
}

#[test]
fn from_database_torrent_info_hash_not_found() {
    let err: TorrentError = database::Error::TorrentInfoHashNotFound.into();
    assert_eq!(err, TorrentError::TorrentNotFound);
}

#[test]
fn from_database_torrent_already_exists() {
    let err: TorrentError = database::Error::TorrentAlreadyExists.into();
    assert_eq!(err, TorrentError::InfoHashAlreadyExists);
}

#[test]
fn from_database_torrent_title_already_exists() {
    let err: TorrentError = database::Error::TorrentTitleAlreadyExists.into();
    assert_eq!(err, TorrentError::TorrentTitleAlreadyExists);
}

#[test]
fn from_database_category_not_found() {
    let err: TorrentError = database::Error::CategoryNotFound.into();
    assert_eq!(err, TorrentError::InvalidCategory);
}

#[test]
fn from_database_tag_not_found() {
    let err: TorrentError = database::Error::TagNotFound.into();
    assert_eq!(err, TorrentError::InvalidTag);
}

#[test]
fn from_database_fallback() {
    let err: TorrentError = database::Error::Error.into();
    assert_eq!(err, TorrentError::DatabaseError);
}

// From<MetadataError>

#[test]
fn from_metadata_missing_title() {
    let err: TorrentError = MetadataError::MissingTorrentTitle.into();
    assert_eq!(err, TorrentError::MissingMandatoryMetadataFields);
}

#[test]
fn from_metadata_invalid_title_length() {
    let err: TorrentError = MetadataError::InvalidTorrentTitleLength.into();
    assert_eq!(err, TorrentError::InvalidTorrentTitleLength);
}

// From<DecodeTorrentFileError>

#[test]
fn from_decode_invalid_bencode() {
    let err: TorrentError = DecodeTorrentFileError::InvalidBencodeData.into();
    assert_eq!(err, TorrentError::InvalidTorrentFile);
}

#[test]
fn from_decode_invalid_info_dict() {
    let err: TorrentError = DecodeTorrentFileError::InvalidInfoDictionary.into();
    assert_eq!(err, TorrentError::InvalidTorrentFile);
}

#[test]
fn from_decode_invalid_pieces_length() {
    let err: TorrentError = DecodeTorrentFileError::InvalidTorrentPiecesLength.into();
    assert_eq!(err, TorrentError::InvalidTorrentPiecesLength);
}

#[test]
fn from_decode_cannot_bencode_info() {
    let err: TorrentError = DecodeTorrentFileError::CannotBencodeInfoDict.into();
    assert_eq!(err, TorrentError::InvalidTorrentFile);
}

// From<TrackerAPIError>

#[test]
fn from_tracker_api_offline() {
    let err: TorrentError = TrackerAPIError::TrackerOffline {
        error: "connection refused".to_string(),
    }
    .into();
    assert_eq!(err, TorrentError::TrackerOffline);
}

#[test]
fn from_tracker_api_internal_server_error() {
    let err: TorrentError = TrackerAPIError::InternalServerError.into();
    assert_eq!(err, TorrentError::TrackerResponseError);
}

#[test]
fn from_tracker_api_not_found() {
    let err: TorrentError = TrackerAPIError::NotFound.into();
    assert_eq!(err, TorrentError::TrackerResponseError);
}

#[test]
fn from_tracker_api_torrent_not_found() {
    let err: TorrentError = TrackerAPIError::TorrentNotFound.into();
    assert_eq!(err, TorrentError::TorrentNotFoundInTracker);
}

#[test]
fn from_tracker_api_unexpected_response() {
    let err: TorrentError = TrackerAPIError::UnexpectedResponseStatus.into();
    assert_eq!(err, TorrentError::TrackerUnknownResponse);
}

#[test]
fn from_tracker_api_missing_response_body() {
    let err: TorrentError = TrackerAPIError::MissingResponseBody.into();
    assert_eq!(err, TorrentError::TrackerUnknownResponse);
}

#[test]
fn from_tracker_api_failed_to_parse() {
    let err: TorrentError = TrackerAPIError::FailedToParseTrackerResponse {
        body: "bad body".to_string(),
    }
    .into();
    assert_eq!(err, TorrentError::TrackerUnknownResponse);
}

#[test]
fn from_tracker_api_cannot_save_user_key() {
    let err: TorrentError = TrackerAPIError::CannotSaveUserKey.into();
    assert_eq!(err, TorrentError::DatabaseError);
}

#[test]
fn from_tracker_api_invalid_token() {
    let err: TorrentError = TrackerAPIError::InvalidToken.into();
    assert_eq!(err, TorrentError::InvalidTrackerToken);
}

// From<std::io::Error>

#[test]
fn from_io_error() {
    let err: TorrentError = io::Error::new(io::ErrorKind::NotFound, "file not found").into();
    assert_eq!(err, TorrentError::InternalServerError);
}

// From<serde_json::Error>

#[test]
fn from_serde_json_error() {
    let source: serde_json::Error = serde_json::from_str::<String>("not json").unwrap_err();
    let err: TorrentError = source.into();
    assert_eq!(err, TorrentError::InternalServerError);
}

// From<Box<dyn std::error::Error>>

#[test]
fn from_boxed_error() {
    let source: Box<dyn std::error::Error> = "some error".into();
    let err: TorrentError = source.into();
    assert_eq!(err, TorrentError::InternalServerError);
}
