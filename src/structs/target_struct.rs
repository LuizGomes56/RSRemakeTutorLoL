use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    game_struct::GameCoreStats,
    riot_champion_struct::{RiotChampionPassive, RiotChampionStats},
    riot_items_struct::{RiotItemGold, RiotItemStats},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RiotChampionTargetSpell {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cooldown: Vec<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RiotChampionTarget {
    pub id: String,
    pub name: String,
    pub stats: RiotChampionStats,
    pub spells: Vec<RiotChampionTargetSpell>,
    pub passive: RiotChampionPassive,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RiotItemTarget {
    pub name: String,
    pub description: String,
    pub stats: RiotItemStats,
    pub gold: RiotItemGold,
    pub maps: HashMap<String, bool>,
    pub from: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AllStatsMultiplier {
    pub magic: f64,
    pub physical: f64,
    pub general: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum AllStatsAdaptativeType {
    Physical,
    Magic,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AllStatsAdaptative {
    pub adaptative_type: AllStatsAdaptativeType,
    pub ratio: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AllStatsChampionStats {
    pub max_health: f64,
    pub armor: f64,
    pub magic_resist: f64,
    pub attack_damage: f64,
    pub resource_max: f64,
    pub ability_power: f64,
    pub current_health: f64,
    pub attack_speed: f64,
    pub attack_range: f64,
    pub crit_chance: f64,
    pub crit_damage: f64,
    pub physical_lethality: f64,
    pub armor_penetration_percent: f64,
    pub magic_penetration_percent: f64,
    pub magic_penetration_flat: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AllStatsActivePlayer {
    pub id: String,
    pub level: u8,
    pub form: AllStatsForm,
    pub multiplier: AllStatsMultiplier,
    pub adaptative: AllStatsAdaptative,
    pub champion_stats: AllStatsChampionStats,
    pub base_stats: GameCoreStats,
    pub bonus_stats: GameCoreStats,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum AllStatsForm {
    Melee,
    Ranged,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AllStatsRealStats {
    pub armor: f64,
    pub magic_resist: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AllStatsPlayer {
    pub multiplier: AllStatsMultiplier,
    pub real_stats: AllStatsRealStats,
    pub champion_stats: GameCoreStats,
    pub base_stats: GameCoreStats,
    pub bonus_stats: GameCoreStats,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AllStatsProperty {
    pub over_health: f64,
    pub missing_health: f64,
    pub excess_health: f64,
    pub steelcaps: f64,
    pub rocksolid: f64,
    pub randuin: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TargetAllStats {
    pub active_player: AllStatsActivePlayer,
    pub player: AllStatsPlayer,
    pub property: AllStatsProperty,
}
