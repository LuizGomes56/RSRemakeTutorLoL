use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::structs::target_struct::RiotChampionTarget;

use super::riot_champion_struct::RiotChampionStats;

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

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GameChampionStats {
    pub ability_power: f64,
    pub armor: f64,
    pub armor_penetration_flat: f64,
    pub armor_penetration_percent: f64,
    pub attack_damage: f64,
    pub attack_range: f64,
    pub crit_chance: f64,
    pub crit_damage: f64,
    pub current_health: f64,
    pub magic_penetration_flat: f64,
    pub magic_penetration_percent: f64,
    pub magic_resist: f64,
    pub max_health: f64,
    pub physical_lethality: f64,
    pub resource_max: f64,
}

impl GameChampionStats {
    fn to_camel_case(snake_str: &str) -> String {
        let mut s = snake_str.split('_').peekable();
        let mut camel_case = String::new();
        if let Some(first) = s.next() {
            camel_case.push_str(first);
        }
        while let Some(word) = s.next() {
            camel_case.push_str(&Self::capitalize(word));
        }
        camel_case
    }
    fn capitalize(word: &str) -> String {
        let mut c = word.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }
    pub fn into_hashmap_camel(&mut self) -> HashMap<String, f64> {
        let json = serde_json::to_value(self).unwrap();
        let mut map = HashMap::new();

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if let Value::Number(num) = value {
                    if let Some(f) = num.as_f64() {
                        let camel_case_key = Self::to_camel_case(&key);
                        map.insert(camel_case_key, f);
                    }
                }
            }
        }
        map
    }
    pub fn from_hashmap_camel(map: HashMap<String, f64>) -> Self {
        let mut stats = Self::default();
        for (key, value) in map {
            match key.as_str() {
                "abilityPower" => stats.ability_power = value,
                "armor" => stats.armor = value,
                "armorPenetrationFlat" => stats.armor_penetration_flat = value,
                "armorPenetrationPercent" => stats.armor_penetration_percent = value,
                "attackDamage" => stats.attack_damage = value,
                "attackRange" => stats.attack_range = value,
                "critChance" => stats.crit_chance = value,
                "critDamage" => stats.crit_damage = value,
                "currentHealth" => stats.current_health = value,
                "magicPenetrationFlat" => stats.magic_penetration_flat = value,
                "magicPenetrationPercent" => stats.magic_penetration_percent = value,
                "magicResist" => stats.magic_resist = value,
                "maxHealth" => stats.max_health = value,
                "physicalLethality" => stats.physical_lethality = value,
                "resourceMax" => stats.resource_max = value,
                _ => {}
            }
        }
        stats
    }
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

impl GameCoreStats {
    fn formula(base: f64, per_level: f64, level: f64) -> f64 {
        base + per_level * (level - 1.0) * (0.7025 + 0.0175 * (level - 1.0))
    }
    pub fn base_stats(stats: &RiotChampionStats, level: u8) -> Self {
        let lvl = level as f64;
        Self {
            max_health: Self::formula(stats.hp, stats.hpperlevel, lvl),
            armor: Self::formula(stats.armor, stats.armorperlevel, lvl),
            magic_resist: Self::formula(stats.spellblock, stats.spellblockperlevel, lvl),
            attack_damage: Self::formula(stats.attackdamage, stats.attackdamageperlevel, lvl),
            resource_max: Self::formula(stats.mp, stats.mpperlevel, lvl),
            ability_power: 0.0,
        }
    }
    pub fn bonus_stats(&self, current: &GameCoreStats) -> Self {
        Self {
            max_health: self.max_health - current.max_health,
            armor: self.armor - current.armor,
            magic_resist: self.magic_resist - current.magic_resist,
            attack_damage: self.attack_damage - current.attack_damage,
            resource_max: self.resource_max - current.resource_max,
            ability_power: 0.0,
        }
    }
}

impl GameChampionStats {
    pub fn bonus_stats(&self, current: GameCoreStats) -> GameCoreStats {
        GameCoreStats {
            max_health: current.max_health - self.max_health,
            armor: current.armor - self.armor,
            magic_resist: current.magic_resist - self.magic_resist,
            attack_damage: current.attack_damage - self.attack_damage,
            resource_max: current.resource_max - self.resource_max,
            ability_power: self.ability_power,
        }
    }
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
    pub name: String,
    pub active: bool,
    pub gold: Option<u32>,
    pub raw: HashMap<String, Value>,
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GamePlayerItems {
    #[serde(rename = "itemID")]
    pub item_id: u32,
}

#[derive(Debug, Clone, Deserialize, Default, Serialize)]
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

impl GamePlayerDamage {
    pub fn void() -> Self {
        Self {
            min: 0.0,
            max: None,
            damage_type: String::from("mixed"),
            ..Default::default()
        }
    }
}

pub type GameDamageReturn = HashMap<String, GamePlayerDamage>;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamePlayerDamages {
    pub abilities: GameDamageReturn,
    pub items: GameDamageReturn,
    pub runes: GameDamageReturn,
    pub spell: GameDamageReturn,
}

impl GamePlayerDamages {
    pub fn into_hashmap(&self) -> HashMap<&'static str, &GameDamageReturn> {
        let mut map = HashMap::new();
        map.insert("abilities", &self.abilities);
        map.insert("items", &self.items);
        map.insert("runes", &self.runes);
        map.insert("spell", &self.spell);
        map
    }
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
