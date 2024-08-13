use serde::Deserialize;

use super::Settings;

#[derive(Deserialize)]
pub struct AllSettingsResponse {
    pub data: Settings,
}

#[derive(Deserialize)]
pub struct PublicSettingsResponse {
    pub data: Public,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Public {
    pub website_name: String,
    pub tracker_url: String,
    pub tracker_mode: String,
    pub email_on_signup: String,
}

#[derive(Deserialize)]
pub struct SiteNameResponse {
    pub data: String,
}
