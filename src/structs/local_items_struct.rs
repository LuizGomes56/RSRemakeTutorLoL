use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct LocalItemForm {
    pub melee: String,
    pub ranged: String,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct DamageRange {
    pub min: LocalItemForm,
    pub max: Option<LocalItemForm>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct LocalItemData {
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub min: LocalItemForm,
    pub max: Option<LocalItemForm>,
    pub onhit: bool,
    pub effect: Option<Vec<f64>>,
    pub damage: Option<DamageRange>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct LocalItems {
    pub data: HashMap<String, LocalItemData>,
}
