use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct LocalStats {
    pub name: String,
    pub stats: HashMap<String, Value>,
    pub stack: bool,
    pub gold: LocalStatsGold,
    pub maps: HashMap<String, bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LocalStatsGold {
    base: u32,
    purchasable: bool,
    total: u32,
    sell: u32,
}
