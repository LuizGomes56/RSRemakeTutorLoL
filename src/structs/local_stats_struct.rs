use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type LocalStats = HashMap<String, LocalStatsStruct>;

#[derive(Debug, Deserialize, Serialize)]
pub struct LocalStatsStruct {
    pub name: String,
    pub stats: LocalStatsHashMap,
    pub stack: bool,
    pub gold: LocalStatsGold,
    pub maps: HashMap<String, bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LocalStatsHashMap {
    pub raw: HashMap<String, Value>,
    #[serde(rename = "mod")]
    pub modifiers: HashMap<String, Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LocalStatsGold {
    base: u32,
    purchasable: bool,
    total: u32,
    sell: u32,
}
