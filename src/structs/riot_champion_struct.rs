use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RiotChampionImage {
    pub full: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RiotChampionStats {
    pub hp: f64,
    pub hpperlevel: f64,
    pub mp: f64,
    pub mpperlevel: f64,
    pub armor: f64,
    pub armorperlevel: f64,
    pub spellblock: f64,
    pub spellblockperlevel: f64,
    pub attackrange: f64,
    pub attackdamage: f64,
    pub attackdamageperlevel: f64,
    pub attackspeedperlevel: f64,
    pub attackspeed: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RiotChampionPassive {
    pub name: String,
    pub description: String,
    pub image: RiotChampionImage,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RiotChampionSpell {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cooldown: Vec<f64>,
    pub image: RiotChampionImage,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RiotChampionSkin {
    pub num: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RiotChampionData {
    pub id: String,
    pub name: String,
    pub image: RiotChampionImage,
    pub skins: Vec<RiotChampionSkin>,
    pub stats: RiotChampionStats,
    pub spells: Vec<RiotChampionSpell>,
    pub passive: RiotChampionPassive,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RiotChampion {
    pub data: HashMap<String, RiotChampionData>,
}
