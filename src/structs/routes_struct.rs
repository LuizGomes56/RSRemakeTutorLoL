use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LastByCodeResponseData {
    pub game_id: String,
    pub summoner_name: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub game_code: Option<String>,
    pub champion_name: Option<String>,
    pub game: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LastByCodeResponse {
    pub success: bool,
    pub data: LastByCodeResponseData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HTTPErrorResponse {
    pub success: bool,
    pub message: &'static str,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LastByCodeRequest {
    pub code: String,
    pub item: String,
    pub rec: bool,
}
