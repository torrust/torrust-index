//! Tests for the domain error system (§T-006).
//!
//! # Index
//!
//! ## `auth_error` (status codes, display, `From` impls)
//!
//! - `wrong_password_or_username_returns_forbidden`
//! - `invalid_password_returns_forbidden`
//! - `username_not_found_returns_not_found`
//! - `token_not_found_returns_unauthorized`
//! - `token_expired_returns_unauthorized`
//! - `token_invalid_returns_unauthorized`
//! - `unauthorized_action_returns_forbidden`
//! - `unauthorized_action_for_guests_returns_unauthorized`
//! - `logged_in_user_not_found_returns_unauthorized`
//! - `email_not_verified_returns_forbidden`
//! - `internal_server_error_returns_500`
//! - `database_error_returns_500`
//! - `user_not_found_returns_not_found`
//! - `display_wrong_password_or_username`
//! - `display_invalid_password`
//! - `display_username_not_found`
//! - `display_token_not_found`
//! - `display_token_expired`
//! - `display_token_invalid`
//! - `display_unauthorized_action`
//! - `display_unauthorized_action_for_guests`
//! - `display_logged_in_user_not_found`
//! - `display_email_not_verified`
//! - `display_internal_server_error`
//! - `display_database_error`
//! - `display_user_not_found`
//! - `from_database_user_not_found`
//! - `from_database_fallback`
//! - `from_argon2_error`
//!
//! ## `user_error` (status codes, display, `From` impls)
//!
//! - `closed_for_registration_returns_forbidden`
//! - `email_missing_returns_not_found`
//! - `email_invalid_returns_bad_request`
//! - `email_taken_returns_bad_request`
//! - `email_not_verified_returns_forbidden`
//! - `failed_to_send_verification_email_returns_500`
//! - `user_not_found_returns_not_found`
//! - `account_not_found_returns_not_found`
//! - `username_taken_returns_bad_request`
//! - `username_invalid_returns_bad_request`
//! - `profanity_error_returns_bad_request`
//! - `blacklist_error_returns_bad_request`
//! - `username_case_mapped_error_returns_bad_request`
//! - `password_too_short_returns_bad_request`
//! - `password_too_long_returns_bad_request`
//! - `passwords_dont_match_returns_bad_request`
//! - `invalid_user_listing_returns_bad_request`
//! - `invalid_password_returns_forbidden`
//! - `unauthorized_action_returns_forbidden`
//! - `unauthorized_action_for_guests_returns_unauthorized`
//! - `database_error_returns_500`
//! - `internal_server_error_returns_500`
//! - `from_auth_unauthorized_action`
//! - `from_auth_unauthorized_action_for_guests`
//! - `from_auth_invalid_password`
//! - `from_auth_fallback`
//! - `from_database_username_taken`
//! - `from_database_email_taken`
//! - `from_database_user_not_found`
//! - `from_database_fallback`
//! - `from_argon2_error`
//! - `from_serde_json_error`
//!
//! ## `torrent_error` (status codes, display, `From` impls)
//!
//! - status-code tests for all 24 variants
//! - display tests for all 24 variants
//! - `from_auth_unauthorized_action`
//! - `from_auth_unauthorized_action_for_guests`
//! - `from_auth_fallback`
//! - `from_database_torrent_not_found`
//! - `from_database_torrent_info_hash_not_found`
//! - `from_database_torrent_already_exists`
//! - `from_database_torrent_title_already_exists`
//! - `from_database_category_not_found`
//! - `from_database_tag_not_found`
//! - `from_database_fallback`
//! - `from_metadata_missing_title`
//! - `from_metadata_invalid_title_length`
//! - `from_decode_invalid_bencode`
//! - `from_decode_invalid_info_dict`
//! - `from_decode_invalid_pieces_length`
//! - `from_decode_cannot_bencode_info`
//! - `from_tracker_api_offline`
//! - `from_tracker_api_internal_server_error`
//! - `from_tracker_api_not_found`
//! - `from_tracker_api_torrent_not_found`
//! - `from_tracker_api_unexpected_response`
//! - `from_tracker_api_missing_response_body`
//! - `from_tracker_api_failed_to_parse`
//! - `from_tracker_api_cannot_save_user_key`
//! - `from_tracker_api_invalid_token`
//! - `from_io_error`
//! - `from_serde_json_error`
//! - `from_boxed_error`
//!
//! ## `category_tag_error` (status codes, display, `From` impls)
//!
//! - status-code tests for all 12 variants
//! - display tests for all 12 variants
//! - `from_auth_unauthorized_action`
//! - `from_auth_unauthorized_action_for_guests`
//! - `from_auth_fallback`
//!
//! ## `api_error` (delegation)
//!
//! - `delegates_status_code_to_auth`
//! - `delegates_status_code_to_user`
//! - `delegates_status_code_to_torrent`
//! - `delegates_status_code_to_category_tag`
//! - `delegates_display_to_auth`
//! - `delegates_display_to_user`
//! - `delegates_display_to_torrent`
//! - `delegates_display_to_category_tag`

mod api_error;
mod auth_error;
mod category_tag_error;
mod torrent_error;
mod user_error;
