use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::structs::target_struct::RiotChampionTarget;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamePassive {
    pub display_name: String,
    pub id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameAbility {
    pub ability_level: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct GameAbilities {
    pub passive: GamePassive,
    pub q: GameAbility,
    pub w: GameAbility,
    pub e: GameAbility,
    pub r: GameAbility,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameChampionStats {
    pub ability_haste: f64,
    pub ability_power: f64,
    pub armor: f64,
    pub armor_penetration_flat: f64,
    pub armor_penetration_percent: f64,
    pub attack_damage: f64,
    pub attack_range: f64,
    pub attack_speed: f64,
    pub bonus_armor_penetration_percent: f64,
    pub bonus_magic_penetration_percent: f64,
    pub crit_chance: f64,
    pub crit_damage: f64,
    pub current_health: f64,
    pub heal_shield_power: f64,
    pub health_regen_rate: f64,
    pub life_steal: f64,
    pub magic_lethality: f64,
    pub magic_penetration_flat: f64,
    pub magic_penetration_percent: f64,
    pub magic_resist: f64,
    pub max_health: f64,
    pub move_speed: f64,
    pub omnivamp: f64,
    pub physical_lethality: f64,
    pub physical_vamp: f64,
    pub resource_max: f64,
    pub resource_regen_rate: f64,
    pub resource_type: String,
    pub resource_value: f64,
    pub spell_vamp: f64,
    pub tenacity: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRuneProp {
    pub display_name: String,
    pub id: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameFullRunes {
    pub general_runes: Vec<GameRuneProp>,
}

#[derive(Debug, Clone, Deserialize, Default, Serialize, Copy)]
#[serde(rename_all = "camelCase")]
pub struct GameCoreStats {
    pub max_health: f64,
    pub armor: f64,
    pub magic_resist: f64,
    pub attack_damage: f64,
    pub resource_max: f64,
    pub ability_power: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRelevantProps {
    pub min: Vec<String>,
    pub max: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameRelevant {
    pub abilities: GameRelevantProps,
    pub items: GameRelevantProps,
    pub runes: GameRelevantProps,
    pub spell: GameRelevantProps,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameToolInfo {
    pub id: String,
    pub name: Option<String>,
    pub active: bool,
    pub gold: Option<f64>,
    pub raw: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameDragonProps {
    pub earth: f64,
    pub fire: f64,
    pub chemtech: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameActivePlayer {
    pub summoner_name: String,
    pub level: u8,
    pub abilities: GameAbilities,
    pub champion_stats: GameChampionStats,
    pub full_runes: GameFullRunes,
    /** Extends Active Player */
    pub champion_name: Option<String>,
    pub champion: Option<RiotChampionTarget>,
    pub dragon: Option<GameDragonProps>,
    pub items: Option<Vec<String>>,
    pub base_stats: Option<GameCoreStats>,
    pub bonus_stats: Option<GameCoreStats>,
    pub team: Option<String>,
    pub skin: Option<u8>,
    pub tool: Option<GameToolInfo>,
    pub relevant: Option<GameRelevant>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameScores {
    pub assists: i32,
    pub kills: i32,
    pub deaths: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSummonerSpell {
    pub display_name: String,
    pub raw_description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSummonerSpells {
    pub summoner_spell_one: GameSummonerSpell,
    pub summoner_spell_two: GameSummonerSpell,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamePlayerItems {
    #[serde(rename = "itemID")]
    pub item_id: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamePlayerDamage {
    pub min: f64,
    pub max: Option<f64>,
    #[serde(rename = "type")]
    pub damage_type: String,
    pub name: Option<String>,
    pub area: Option<bool>,
    pub onhit: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamePlayerDamages {
    pub abilities: HashMap<String, GamePlayerDamage>,
    pub items: HashMap<String, GamePlayerDamage>,
    pub runes: HashMap<String, GamePlayerDamage>,
    pub spell: HashMap<String, GamePlayerDamage>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GamePlayerTool {
    pub dif: Option<GamePlayerDamages>,
    pub max: GamePlayerDamages,
    pub sum: f64,
    pub rec: Option<HashMap<String, f64>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamePlayer {
    pub champion_name: String,
    pub level: u8,
    pub position: String,
    pub summoner_name: String,
    pub scores: GameScores,
    pub items: Vec<GamePlayerItems>,
    pub summoner_spells: GameSummonerSpells,
    #[serde(rename = "skinID")]
    pub skin_id: u8,
    pub team: String,
    /** Extends Player */
    pub champion: Option<RiotChampionTarget>,
    pub dragon: Option<GameDragonProps>,
    pub bonus_stats: Option<GameCoreStats>,
    pub base_stats: Option<GameCoreStats>,
    pub champion_stats: Option<GameCoreStats>,
    pub damage: Option<GamePlayerDamages>,
    pub tool: Option<GamePlayerTool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameData {
    pub game_time: f64,
    pub map_number: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct GameEventProps {
    pub event_name: String,
    pub killer_name: Option<String>,
    pub dragon_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct GameEvents {
    pub events: Vec<GameEventProps>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameProps {
    pub active_player: GameActivePlayer,
    pub all_players: Vec<GamePlayer>,
    pub events: GameEvents,
    pub game_data: GameData,
}
