use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::sync::RwLock as RwLockAsync;
use tokio::task;

use futures::stream::FuturesUnordered;
use futures::stream::StreamExt;

use meval::eval_str;
use regex::Regex;

use super::lol_service::{champion_api, item_api};
use crate::fetch_json_sync;
use crate::structs::game_struct::GameAbilities;
use crate::structs::game_struct::GameDamageReturn;
use crate::structs::game_struct::GamePlayerDamage;
use crate::structs::game_struct::GamePlayerDamages;
use crate::structs::local_champion_struct::LocalChampionAbility;
use crate::structs::target_struct::TargetReplacements;
use crate::{
    fetch_json,
    structs::{
        game_struct::{
            GameActivePlayer, GameChampionStats, GameCoreStats, GameFullRunes, GamePlayer,
            GameProps, GameRelevant, GameRelevantProps, GameSummonerSpells,
        },
        local_champion_struct::LocalChampion,
        local_items_struct::LocalItems,
        local_runes_struct::LocalRunes,
        target_struct::{
            AllStatsActivePlayer, AllStatsAdaptative, AllStatsChampionStats, AllStatsMultiplier,
            AllStatsPlayer, AllStatsProperty, AllStatsRealStats, TargetAllStats,
        },
    },
};

static LOCAL_ITEMS: Lazy<Arc<LocalItems>> = Lazy::new(|| {
    Arc::new(fetch_json_sync::<LocalItems>("src/effects/items").expect("Falha ao carregar itens"))
});

static LOCAL_RUNES: Lazy<Arc<LocalRunes>> = Lazy::new(|| {
    Arc::new(fetch_json_sync::<LocalRunes>("src/effects/runes").expect("Falha ao carregar runas"))
});

static LAST_CHAMP_ID: Lazy<RwLock<Option<String>>> = Lazy::new(|| RwLock::new(None));

static LOCAL_CHAMP: Lazy<RwLock<LocalChampion>> =
    Lazy::new(|| RwLock::new(HashMap::<String, LocalChampionAbility>::new()));

/*
async fn assign_champion(g: &mut GameProps) {
    for player in &mut g.all_players.iter_mut() {
        if player.summoner_name == g.active_player.summoner_name {
            g.active_player.team = Some(player.team.clone());
        }
        let c = champion_api(&player.champion_name).await;
        match c {
            Ok(c) => player.champion = Some(c),
            Err(e) => panic!("Error fetching champion in assign_champion(): {:?}", e),
        }
    }
}
*/

/*
async fn assign_champion(data: GameProps) -> GameProps {
    let mut futures = FuturesUnordered::new();
    let data_arc = Arc::new(RwLock::new(data));
    {
        let data_read = data_arc.read().unwrap();
        for player_iter in data_read.all_players.iter() {
            let data_clone = Arc::clone(&data_arc);
            let player_name = player_iter.summoner_name.clone();

            futures.push(async move {
                let c = champion_api(&player_name).await;
                match c {
                    Ok(champion_data) => {
                        let mut data_write = data_clone.write().unwrap();
                        let player = data_write
                            .all_players
                            .iter_mut()
                            .find(|p| p.summoner_name == player_name)
                            .unwrap();
                        player.champion = Some(champion_data);
                    }
                    Err(e) => panic!("Error fetching champion in assign_champion(): {:?}", e),
                }
            });
        }
    }
    while let Some(_) = futures.next().await {}
    Arc::try_unwrap(data_arc).unwrap().into_inner().unwrap()
}
*/
async fn assign_champion(data: GameProps) -> GameProps {
    let data_arc = Arc::new(RwLockAsync::new(data));
    let mut futures = FuturesUnordered::new();

    {
        let data_read = data_arc.read().await;
        for player_iter in data_read.all_players.iter() {
            let data_clone = Arc::clone(&data_arc);
            let player_name = player_iter.summoner_name.clone();
            let player_champ = player_iter.champion_name.clone();

            futures.push(async move {
                let c = champion_api(&player_champ).await;
                if let Ok(champion_data) = c {
                    let mut data_write = data_clone.write().await;
                    if let Some(player) = data_write
                        .all_players
                        .iter_mut()
                        .find(|p| p.summoner_name == player_name)
                    {
                        player.champion = Some(champion_data);
                    }
                }
            });
        }
    }

    while let Some(_) = futures.next().await {}
    Arc::try_unwrap(data_arc).unwrap().into_inner()
}

pub async fn calculate(mut data: GameProps) -> () {
    data = assign_champion(data).await;

    let mut active_player = Arc::new(data.active_player);
    let all_players = data.all_players;

    for player in all_players.iter() {
        if player.summoner_name == active_player.summoner_name {
            if let Some(champion) = &player.champion {
                {
                    let last_champ_id = LAST_CHAMP_ID.read().unwrap();
                    if last_champ_id.as_ref() != Some(&champion.id) {
                        drop(last_champ_id);

                        let champ =
                            fetch_json::<LocalChampion>(&format!("src/champions/{}", &champion.id))
                                .await
                                .unwrap();

                        let mut local_champ = LOCAL_CHAMP.write().unwrap();
                        let mut last_champ_id = LAST_CHAMP_ID.write().unwrap();

                        *local_champ = champ;
                        *last_champ_id = Some(champion.id.clone());
                    }
                }

                let acp = Arc::make_mut(&mut active_player);

                acp.team = Some(player.team.clone());
                acp.champion = Some(champion.clone());
                acp.champion_name = Some(champion.name.clone());
                acp.skin = Some(player.skin_id);

                acp.base_stats = Some(GameCoreStats::base_stats(&champion.stats, player.level));
                acp.bonus_stats = Some(GameChampionStats::bonus_stats(
                    &acp.champion_stats,
                    acp.base_stats.unwrap(),
                ));

                acp.relevant = Some(GameRelevant {
                    abilities: filter_abilities(&LOCAL_CHAMP.read().unwrap()),
                    items: filter_items(
                        &LOCAL_ITEMS,
                        &player
                            .items
                            .iter()
                            .map(|item| item.item_id.to_string())
                            .collect::<Vec<String>>(),
                    ),
                    runes: filter_runes(&LOCAL_RUNES, &acp.full_runes),
                    spell: filter_spell(&player.summoner_spells),
                });
            }
        }
    }

    let mut futures = FuturesUnordered::new();

    for mut player in all_players.into_iter() {
        let active_player_clone = Arc::clone(&active_player);

        if &player.team.as_str() != &active_player_clone.team.as_ref().unwrap() {
            futures.push(async move {
                if let Some(champion) = &player.champion {
                    player.base_stats =
                        Some(GameCoreStats::base_stats(&champion.stats, player.level));

                    let items: Vec<String> = player
                        .items
                        .iter()
                        .map(|item| item.item_id.to_string())
                        .collect();

                    player.champion_stats =
                        Some(player_stats(player.base_stats.unwrap(), items).await);

                    player.bonus_stats = Some(GameCoreStats::bonus_stats(
                        &player.champion_stats.unwrap(),
                        &player.base_stats.unwrap(),
                    ));

                    let stats = all_stats(&player, &active_player_clone);

                    player.damage = Some(GamePlayerDamages {
                        abilities: ability_damage(
                            &stats,
                            &active_player_clone.abilities,
                            &LOCAL_CHAMP.read().unwrap(),
                        ),
                        items: item_damage(
                            &stats,
                            &active_player_clone.relevant.as_ref().unwrap().items.min,
                            &LOCAL_ITEMS,
                        ),
                        runes: rune_damage(
                            &stats,
                            &active_player_clone.relevant.as_ref().unwrap().runes.min,
                            &LOCAL_RUNES,
                        ),
                        spell: spell_damage(
                            &active_player_clone.relevant.as_ref().unwrap().spell.min,
                            active_player_clone.level,
                        ),
                    })
                }
                player
            });
        }
    }
    let mut all_players_collected = Vec::<GamePlayer>::with_capacity(5);

    while let Some(t) = futures.next().await {
        all_players_collected.push(t);
    }
}

fn json_replacements(stats: &TargetAllStats) -> TargetReplacements {
    let x = &stats.active_player;
    let y = &stats.player;
    let z = &stats.property;
    let k = &x.champion_stats;
    let t = &x.base_stats;
    let n = &x.bonus_stats;
    let m = &y.champion_stats;

    let entries = [
        ("steelcapsEffect", z.steelcaps),
        ("attackReductionEffect", z.rocksolid),
        ("exceededHP", z.excess_health),
        ("missingHP", z.missing_health),
        ("magicMod", x.multiplier.magic),
        ("physicalMod", x.multiplier.physical),
        ("level", x.level as f64),
        ("currentAP", k.ability_power),
        ("currentAD", k.attack_damage),
        ("currentLethality", k.physical_lethality),
        ("maxHP", k.max_health),
        ("maxMana", k.resource_max),
        ("currentMR", k.magic_resist),
        ("currentArmor", k.armor),
        ("currentHealth", k.current_health),
        ("basicAttack", 1.0),
        ("attackSpeed", k.attack_speed),
        ("critChance", k.crit_chance),
        ("critDamage", k.crit_damage),
        ("adaptative", x.adaptative.ratio),
        ("baseHP", t.max_health),
        ("baseMana", t.resource_max),
        ("baseArmor", t.armor),
        ("baseMR", t.magic_resist),
        ("baseAD", t.attack_damage),
        ("bonusAD", n.attack_damage),
        ("bonusHP", n.max_health),
        ("bonusArmor", n.armor),
        ("bonusMR", n.magic_resist),
        ("expectedHealth", m.max_health),
        ("expectedMana", m.resource_max),
        ("expectedArmor", m.armor),
        ("expectedMR", m.magic_resist),
        ("expectedAD", m.attack_damage),
        ("expectedBonusHealth", y.bonus_stats.max_health),
    ];

    entries.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

fn evaluate(
    min: &String,
    max: Option<&String>,
    rep: &TargetAllStats,
    inc: Option<TargetReplacements>,
) -> (f64, Option<f64>) {
    let mut replacements: TargetReplacements = json_replacements(&rep);
    if let Some(custom_replacements) = inc {
        replacements.extend(custom_replacements);
    }
    fn eval_expression(expr: &str) -> Option<f64> {
        match eval_str(expr) {
            Ok(result) => Some(result),
            Err(_) => None,
        }
    }
    fn result(t: Option<&String>, replacements: &TargetReplacements) -> Option<f64> {
        t.map(|expr| {
            let mut expr = expr.clone();
            for (key, value) in replacements {
                let re = Regex::new(&format!(r"\b{}\b", key)).unwrap();
                expr = re.replace_all(&expr, &value.to_string()).to_string();
            }
            eval_expression(&expr).unwrap_or(0.0)
        })
    }
    let res_min = result(Some(min), &replacements).unwrap();
    let res_max = result(max, &replacements);
    (res_min, res_max)
}

fn rune_damage(
    stats: &TargetAllStats,
    runes: &Vec<String>,
    local_runes: &LocalRunes,
) -> GameDamageReturn {
    let mut result = GameDamageReturn::with_capacity(6);
    let form = &stats.active_player.form;

    for rune in runes {
        let element = local_runes.data.get(rune);
        match element {
            Some(val) => {
                let min_str: &String;
                match form.as_str() {
                    "melee" => {
                        min_str = &val.min.melee;
                    }
                    "ranged" => {
                        min_str = &val.min.ranged;
                    }
                    _ => break,
                }
                let (min, _) = evaluate(min_str, None, stats, None);
                result.insert(
                    rune.to_string(),
                    GamePlayerDamage {
                        min,
                        max: None,
                        damage_type: val.rune_type.to_string(),
                        name: Some(val.name.to_string()),
                        onhit: None,
                        area: None,
                    },
                );
            }
            None => continue,
        }
    }
    result
}

fn item_damage(
    stats: &TargetAllStats,
    items: &Vec<String>,
    local_items: &LocalItems,
) -> GameDamageReturn {
    let mut result = GameDamageReturn::with_capacity(6);
    let form = &stats.active_player.form;

    for item in items {
        let element = local_items.data.get(item);
        match element {
            Some(val) => {
                let min_str: &String;
                let max_str: Option<&String>;
                match form.as_str() {
                    "melee" => {
                        min_str = &val.min.melee;
                        max_str = val.max.as_ref().and_then(|x| Some(&x.melee));
                    }
                    "ranged" => {
                        min_str = &val.min.ranged;
                        max_str = val.max.as_ref().and_then(|x| Some(&x.ranged));
                    }
                    _ => break,
                }
                let mut total: Option<f64> = None;
                if let Some(t) = val.effect.as_ref() {
                    total = Some(t[(stats.active_player.level - 1) as usize]);
                }
                let (min, max) = evaluate(
                    min_str,
                    max_str,
                    stats,
                    if total.is_some() {
                        Some(HashMap::from([("total".to_string(), total.unwrap())]))
                    } else {
                        None
                    },
                );
                result.insert(
                    item.to_string(),
                    GamePlayerDamage {
                        min,
                        max,
                        damage_type: val.item_type.to_string(),
                        name: Some(val.name.to_string()),
                        onhit: Some(val.onhit),
                        area: None,
                    },
                );
            }
            None => continue,
        }
    }
    result
}

fn spell_damage(spells: &Vec<String>, level: u8) -> GameDamageReturn {
    let mut result = GameDamageReturn::with_capacity(1);
    for spell in spells {
        if spell == "SummonerDot" {
            result.insert(
                spell.to_string(),
                GamePlayerDamage {
                    min: 50.0 + 20.0 * level as f64,
                    max: None,
                    damage_type: String::from("true"),
                    name: Some(String::from("Ignite")),
                    onhit: None,
                    area: None,
                },
            );
        }
    }
    result
}

fn ability_damage(
    stats: &TargetAllStats,
    abilities: &GameAbilities,
    local_champ: &LocalChampion,
) -> GameDamageReturn {
    let mut result = GameDamageReturn::with_capacity(8);
    for (key, val) in local_champ {
        let index: usize = match key.chars().next().unwrap() {
            'Q' => (abilities.q.ability_level - 1).into(),
            'W' => (abilities.w.ability_level - 1).into(),
            'E' => (abilities.e.ability_level - 1).into(),
            'R' => (abilities.r.ability_level - 1).into(),
            'P' => (stats.active_player.level - 1).into(),
            _ => panic!("Unknown key: {}", key),
        };
        if index == 0 {
            result.insert(key.to_string(), GamePlayerDamage::void());
        }
        let min_str = &val.min[index];
        let max_str = val.max.as_ref().and_then(|t| t.get(index));

        let (min, max) = evaluate(min_str, max_str, stats, None);

        result.insert(
            key.to_string(),
            GamePlayerDamage {
                min,
                max,
                damage_type: val.ability_type.to_string(),
                name: None,
                area: None,
                onhit: None,
            },
        );
    }
    let acst = &stats.active_player.champion_stats;
    let attack = acst.attack_damage * stats.active_player.multiplier.physical;
    result.insert(
        "A".to_string(),
        GamePlayerDamage {
            min: attack,
            damage_type: String::from("physical"),
            max: None,
            name: None,
            area: None,
            onhit: None,
        },
    );
    result.insert(
        "C".to_string(),
        GamePlayerDamage {
            min: attack * acst.crit_damage / 100.0,
            damage_type: String::from("physical"),
            max: None,
            name: None,
            area: None,
            onhit: None,
        },
    );
    result
}

fn all_stats(player: &GamePlayer, active_player: &GameActivePlayer) -> TargetAllStats {
    let acs = &active_player.champion_stats;
    let abs = &active_player.bonus_stats.unwrap();
    let abt = &active_player.base_stats.unwrap();
    let rel = &active_player.relevant.as_ref().unwrap();

    let pcs = &player.champion_stats.unwrap();
    let pbs = &player.bonus_stats.unwrap();
    let pbt = &player.base_stats.unwrap();

    let mut acp_mod = 1.0;
    let pphy_mod = 1.0;
    let pmag_mod = 1.0;
    let pgen_mod = 1.0;

    let rar = pcs.armor * acs.armor_penetration_percent - acs.armor_penetration_flat;
    let rmr = pcs.magic_resist * acs.magic_penetration_percent - acs.magic_penetration_flat;

    let physical = 100.0 / (100.0 + rar);
    let magic = 100.0 / (100.0 + rmr);

    let adp = 0.35 * abs.attack_damage >= 0.2 * acs.ability_power;
    let add = if adp { physical } else { magic };

    let ohp = pcs.max_health / acs.max_health;
    let ehp = pcs.max_health - acs.max_health;
    let mshp = 1.0 - acs.current_health / acs.max_health;

    let exhp = ehp.clamp(0.0, 2500.0);

    if rel.runes.min.contains(&"8299".to_string()) {
        if mshp > 0.7 {
            acp_mod += 0.11;
        } else if mshp >= 0.4 {
            acp_mod += 0.2 * mshp - 0.03;
        }
    }

    if rel.items.min.contains(&"4015".to_string()) {
        acp_mod += exhp / (220000.0 / 15.0);
    }

    let form = if acs.attack_range > 350.0 {
        "ranged".to_string()
    } else {
        "melee".to_string()
    };

    let adaptative_type = if adp {
        "physical".to_string()
    } else {
        "magic".to_string()
    };

    TargetAllStats {
        active_player: AllStatsActivePlayer {
            id: active_player.champion.as_ref().unwrap().id.clone(),
            level: active_player.level,
            form,
            multiplier: AllStatsMultiplier {
                magic,
                physical,
                general: acp_mod,
            },
            adaptative: AllStatsAdaptative {
                adaptative_type,
                ratio: add,
            },
            champion_stats: AllStatsChampionStats {
                ability_power: acs.ability_power,
                attack_damage: acs.attack_damage,
                magic_resist: acs.magic_resist,
                armor: acs.armor,
                resource_max: acs.resource_max,
                max_health: acs.max_health,
                current_health: acs.current_health,
                attack_speed: acs.attack_speed,
                attack_range: acs.attack_range,
                crit_chance: acs.crit_chance,
                crit_damage: acs.crit_damage,
                physical_lethality: acs.physical_lethality,
                armor_penetration_percent: acs.armor_penetration_percent,
                magic_penetration_percent: acs.magic_penetration_percent,
                magic_penetration_flat: acs.magic_penetration_flat,
            },
            base_stats: GameCoreStats {
                max_health: abt.max_health,
                resource_max: abt.resource_max,
                armor: abt.armor,
                magic_resist: abt.magic_resist,
                attack_damage: abt.attack_damage,
                ability_power: abt.ability_power,
            },
            bonus_stats: GameCoreStats {
                max_health: abs.max_health,
                resource_max: abs.resource_max,
                armor: abs.armor,
                magic_resist: abs.magic_resist,
                attack_damage: abs.attack_damage,
                ability_power: abs.ability_power,
            },
        },
        player: AllStatsPlayer {
            multiplier: AllStatsMultiplier {
                magic: pmag_mod,
                physical: pphy_mod,
                general: pgen_mod,
            },
            real_stats: AllStatsRealStats {
                magic_resist: rmr,
                armor: rar,
            },
            champion_stats: GameCoreStats {
                max_health: pcs.max_health,
                resource_max: pcs.resource_max,
                armor: pcs.armor,
                magic_resist: pcs.magic_resist,
                attack_damage: pcs.attack_damage,
                ability_power: pcs.ability_power,
            },
            base_stats: GameCoreStats {
                max_health: pbt.max_health,
                resource_max: pbt.resource_max,
                armor: pbt.armor,
                magic_resist: pbt.magic_resist,
                attack_damage: pbt.attack_damage,
                ability_power: pbt.ability_power,
            },
            bonus_stats: GameCoreStats {
                max_health: pbs.max_health,
                resource_max: pbs.resource_max,
                armor: pbs.armor,
                magic_resist: pbs.magic_resist,
                attack_damage: pbs.attack_damage,
                ability_power: pbs.ability_power,
            },
        },
        property: AllStatsProperty {
            over_health: if ohp < 1.1 {
                0.65
            } else if ohp > 2.0 {
                2.0
            } else {
                ohp
            },
            missing_health: mshp,
            excess_health: exhp,
            steelcaps: if player
                .items
                .iter()
                .find(|item| item.item_id.to_string() == "3143")
                .is_some()
            {
                0.88
            } else {
                1.0
            },
            rocksolid: player
                .items
                .iter()
                .filter(|item| {
                    ["3143", "3110", "3082"].contains(&item.item_id.to_string().as_str())
                })
                .fold(0.0, |total, _| total + (pcs.max_health / 1000.0 * 3.5)),
            randuin: if player
                .items
                .iter()
                .find(|item| item.item_id.to_string() == "3143")
                .is_some()
            {
                0.7
            } else {
                1.0
            },
        },
    }
}

async fn player_stats(mut base: GameCoreStats, items: Vec<String>) -> GameCoreStats {
    for item in items {
        let res = item_api(&item).await.unwrap();
        let stats = res.stats;
        for (key, val) in stats.iter() {
            match key.as_str() {
                "FlatHPPoolMod" => base.max_health += val,
                "FlatMPPoolMod" => base.resource_max += val,
                "FlatMagicDamageMod" => base.ability_power += val,
                "FlatArmorMod" => base.armor += val,
                "FlatSpellBlockMod" => base.magic_resist += val,
                "FlatPhysicalDamageMod" => base.ability_power += val,
                _ => continue,
            }
        }
    }
    base
}

fn filter_abilities(_champion: &LocalChampion) -> GameRelevantProps {
    let mut min = Vec::with_capacity(8);
    let mut max = Vec::with_capacity(8);
    for (key, val) in _champion.iter() {
        if val.max.is_some() {
            max.push(key.clone());
        }
        min.push(key.clone());
    }
    min.push(String::from("A"));
    min.push(String::from("C"));
    GameRelevantProps { min, max }
}

fn filter_items(_items: &LocalItems, items: &Vec<String>) -> GameRelevantProps {
    let mut min = Vec::with_capacity(6);
    let mut max = Vec::with_capacity(6);
    for (key, val) in _items.data.iter() {
        if items.contains(key) {
            min.push(key.clone());
            if val.max.is_some() {
                max.push(key.clone());
            }
        }
    }
    GameRelevantProps { min, max }
}

fn filter_runes(_runes: &LocalRunes, runes: &GameFullRunes) -> GameRelevantProps {
    let mut min = Vec::with_capacity(6);
    let mut max = Vec::with_capacity(6);
    let rune_vec: Vec<String> = runes
        .general_runes
        .iter()
        .map(|r| r.id.to_string())
        .collect();
    for (key, val) in _runes.data.iter() {
        if rune_vec.contains(key) {
            min.push(key.clone());
            if val.max.is_some() {
                max.push(key.clone());
            }
        }
    }
    GameRelevantProps { min, max }
}

fn filter_spell(spells: &GameSummonerSpells) -> GameRelevantProps {
    let mut min = Vec::with_capacity(1);
    let max = Vec::with_capacity(0);
    let spells_vec: Vec<&str> = vec![
        &spells.summoner_spell_one.raw_description,
        &spells.summoner_spell_two.raw_description,
    ];
    for spell in spells_vec {
        if spell.contains("SummonerDot") {
            min.push(String::from("SummonerDot"));
        }
    }
    GameRelevantProps { min, max }
}
