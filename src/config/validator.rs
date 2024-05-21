//! Trait to validate the whole settings of sections of the settings.
use thiserror::Error;
use torrust_index_located_error::LocatedError;
use url::ParseError;

/// Errors that can occur validating the configuration.
#[derive(Error, Debug)]
pub enum ValidationError {
    /// Unable to load the configuration from the configuration file.
    #[error("Invalid tracker URL: {source}")]
    InvalidTrackerUrl { source: LocatedError<'static, ParseError> },

    #[error("UDP private trackers are not supported. URL schemes for private tracker URLs must be HTTP ot HTTPS")]
    UdpTrackersInPrivateModeNotSupported,
}

pub trait Validator {
    /// # Errors
    ///
    /// Will return an error if the configuration is invalid.
    fn validate(&self) -> Result<(), ValidationError>;
}
