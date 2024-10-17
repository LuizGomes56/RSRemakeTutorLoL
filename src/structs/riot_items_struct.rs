use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Default, Serialize)]
pub struct RiotItemGold {
    pub base: i32,
    pub total: i32,
    pub sell: i32,
    pub purchasable: bool,
}

pub type RiotItemStats = Option<HashMap<String, f64>>;

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RiotItem {
    pub name: Option<String>,
    pub gold: Option<RiotItemGold>,
    pub description: Option<String>,
    pub stats: Option<HashMap<String, f64>>,
    pub maps: Option<HashMap<String, bool>>,
    pub effect: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RiotItems {
    pub data: HashMap<String, RiotItem>,
}
