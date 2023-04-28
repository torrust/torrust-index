use serde::Deserialize;

#[derive(Deserialize)]
pub struct SuccessfulLoginResponse {
    pub data: LoggedInUserData,
}

#[derive(Deserialize, Debug)]
pub struct LoggedInUserData {
    pub token: String,
    pub username: String,
    pub admin: bool,
}

#[derive(Deserialize)]
pub struct TokenVerifiedResponse {
    pub data: String,
}

#[derive(Deserialize)]
pub struct BannedUserResponse {
    pub data: String,
}

#[derive(Deserialize)]
pub struct TokenRenewalResponse {
    pub data: TokenRenewalData,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct TokenRenewalData {
    pub token: String,
    pub username: String,
    pub admin: bool,
}
