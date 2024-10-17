use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct LocalRuneForm {
    pub melee: String,
    pub ranged: String,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct LocalRuneData {
    pub name: String,
    #[serde(rename = "type")]
    pub rune_type: String,
    pub min: LocalRuneForm,
    pub max: Option<LocalRuneForm>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct LocalRunes {
    pub data: HashMap<String, LocalRuneData>,
}
