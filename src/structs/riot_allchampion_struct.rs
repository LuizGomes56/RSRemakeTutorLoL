use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct RiotAllChampion {
    pub data: HashMap<String, RiotAllChampionData>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RiotAllChampionStats {
    pub hp: f64,
    pub hpperlevel: f64,
    pub mp: f64,
    pub mpperlevel: f64,
    pub movespeed: f64,
    pub armor: f64,
    pub armorperlevel: f64,
    pub spellblock: f64,
    pub spellblockperlevel: f64,
    pub attackrange: f64,
    pub hpregen: f64,
    pub hpregenperlevel: f64,
    pub mpregen: f64,
    pub mpregenperlevel: f64,
    pub crit: f64,
    pub critperlevel: f64,
    pub attackdamage: f64,
    pub attackdamageperlevel: f64,
    pub attackspeedperlevel: f64,
    pub attackspeed: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RiotAllChampionData {
    pub id: String,
    pub name: String,
    pub stats: RiotAllChampionStats,
}
