use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub type LocalChampion = HashMap<String, LocalChampionAbility>;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct LocalChampionAbility {
    #[serde(rename = "type")]
    pub ability_type: String,
    pub area: Option<bool>,
    pub min: Vec<String>,
    pub max: Option<Vec<String>>,
}
