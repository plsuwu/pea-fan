use serde::{Deserialize, Serialize};

/// for `update_chatter_in_cache`
#[derive(Debug, Deserialize)]
pub struct AliasUpdateRequest {
    pub current: String,
    pub historic: Vec<String>,
}

/// for anything that requires chatter/channel id input
#[derive(Debug, Deserialize)]
pub struct UserIdRequest {
    pub id: String,
}

/// for anything that requires chatter/channel login input
#[derive(Debug, Deserialize)]
pub struct UserLoginRequest {
    pub login: String,
}

/// for heuristically-determining login or id input
#[derive(Debug, Deserialize)]
pub struct UserRequest {
    pub user: String,
}

/// for `totp_compare`
#[derive(Debug, Deserialize)]
pub struct TOTPRequest {
    pub token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScoreVariant {
    Channel,
    Chatter,
}

#[derive(Debug, Deserialize)]
pub struct ScoreWindowQuery {
    pub variant: ScoreVariant,
}

#[derive(Debug, Serialize)]
pub struct TOTPResponse {
    pub is_valid: bool,
    pub session: String,
}

impl TOTPResponse {
    pub fn new(is_valid: bool, session: String) -> Self {
        Self { is_valid, session }
    }
}
