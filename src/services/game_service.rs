use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::sync::RwLock as RwLockAsync;

use futures::stream::FuturesUnordered;
use futures::stream::StreamExt;

use meval::eval_str;
use regex::Regex;

use super::lol_service::{champion_api, item_api};
use crate::structs::game_struct::GameAbilities;
use crate::structs::game_struct::GameDamageReturn;
use crate::structs::game_struct::GamePlayerDamage;
use crate::structs::game_struct::GamePlayerDamages;
use crate::structs::game_struct::GamePlayerTool;
use crate::structs::game_struct::GameToolInfo;
use crate::structs::local_champion_struct::LocalChampionAbility;
use crate::structs::local_stats_struct::LocalStats;
use crate::structs::target_struct::TargetReplacements;
use crate::structs::target_struct::TargetToolChange;
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
use crate::{fetch_json_sync, structured_clone};

static LOCAL_ITEMS: Lazy<Arc<LocalItems>> = Lazy::new(|| {
    Arc::new(fetch_json_sync::<LocalItems>("src/effects/items").expect("Falha ao carregar itens"))
});

static LOCAL_RUNES: Lazy<Arc<LocalRunes>> = Lazy::new(|| {
    Arc::new(fetch_json_sync::<LocalRunes>("src/effects/runes").expect("Falha ao carregar runas"))
});

static LOCAL_STATS: Lazy<Arc<LocalStats>> = Lazy::new(|| {
    Arc::new(fetch_json_sync::<LocalStats>("src/cache/stats").expect("Falha ao carregar stats"))
});

static LAST_CHAMP_ID: Lazy<RwLock<Option<String>>> = Lazy::new(|| RwLock::new(None));

static LOCAL_CHAMP: Lazy<RwLock<LocalChampion>> =
    Lazy::new(|| RwLock::new(HashMap::<String, LocalChampionAbility>::new()));

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

pub async fn calculate(mut data: GameProps, tool_item: &str) -> GameProps {
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

                {
                    let path = &LOCAL_STATS.get(tool_item).unwrap();
                    let raw = &path.stats.raw;
                    let name = &path.name;
                    let gold = &path.gold.total;

                    acp.tool = Some(GameToolInfo {
                        id: tool_item.to_string(),
                        name: name.clone(),
                        active: LOCAL_ITEMS.data.keys().find(|t| t == &tool_item).is_some(),
                        gold: Some(gold.clone()),
                        raw: raw.clone(),
                    });
                }
            }
        }
    }

    let mut futures = FuturesUnordered::new();

    let active_player_clone = Arc::clone(&active_player);

    for mut player in all_players
        .into_iter()
        .filter(|p| &p.team != active_player_clone.team.as_ref().unwrap())
    {
        let active_player_clone = Arc::clone(&active_player);
        futures.push(async move {
            if let Some(champion) = &player.champion {
                player.base_stats = Some(GameCoreStats::base_stats(&champion.stats, player.level));

                let items: Vec<String> = player
                    .items
                    .iter()
                    .map(|item| item.item_id.to_string())
                    .collect();

                player.champion_stats = Some(player_stats(player.base_stats.unwrap(), items).await);

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
                });
                player.tool = Some(tool_damage(
                    structured_clone(&active_player_clone),
                    &player,
                    tool_item,
                ));
            }
            player
        });
    }
    let mut all_players_collected = Vec::<GamePlayer>::with_capacity(5);

    while let Some(t) = futures.next().await {
        all_players_collected.push(t);
    }

    drop(active_player_clone);

    GameProps {
        active_player: Arc::try_unwrap(active_player).unwrap(),
        all_players: all_players_collected,
        events: data.events,
        game_data: data.game_data,
    }
}

fn assing_stats(item: &str, active_player: &mut GameActivePlayer) -> HashMap<String, f64> {
    let mut stats = active_player.champion_stats.into_hashmap_camel();
    if let Some(item) = &LOCAL_STATS.get(item) {
        let modifiers = &item.stats.modifiers;
        for (key, val) in modifiers.into_iter() {
            if let Some(k) = stats.get_mut(key) {
                match val.to_string().parse::<f64>() {
                    Ok(v) => *k += v,
                    Err(_) => {
                        let v = val.as_str().map(|s| s.replace("%", ""));
                        *k -= v.unwrap().parse::<f64>().unwrap_or(0.0);
                    }
                }
            }
        }
    }
    stats
}

fn evaluate_change(next: &GamePlayerDamage, curr: &GamePlayerDamage) -> GamePlayerDamage {
    GamePlayerDamage {
        min: next.min - curr.min,
        max: if next.max.is_some() && curr.max.is_some() {
            Some(next.max.unwrap() - curr.max.unwrap())
        } else {
            None
        },
        damage_type: next.damage_type.clone(),
        name: next.name.clone(),
        area: next.area,
        onhit: next.onhit,
    }
}

fn process_change(
    at: &str,
    val: &HashMap<String, GamePlayerDamage>,
    min_at: &HashMap<String, GamePlayerDamage>,
    change_at: &mut HashMap<String, GamePlayerDamage>,
    sum: &mut f64,
) {
    for (k, v) in val.iter() {
        match min_at.get(k.as_str()) {
            Some(curr) => {
                let result = evaluate_change(v, curr);
                *sum += result.min;
                if let Some(max) = result.max {
                    *sum += max;
                }
                change_at.insert(k.to_owned(), result);
            }
            None => {
                println!(
                    "Key is {}, and was not found in both directions on {}",
                    k, at
                );
                continue;
            }
        }
    }
}

fn find_change(
    max: &GamePlayerDamages,
    min: &GamePlayerDamages,
    sum: &mut f64,
) -> GamePlayerDamages {
    let mut change = GamePlayerDamages {
        abilities: GameDamageReturn::new(),
        items: GameDamageReturn::new(),
        runes: GameDamageReturn::new(),
        spell: GameDamageReturn::new(),
    };
    let max_hashmap = max.into_hashmap();
    let min_hashmap = min.into_hashmap();
    for (key, val) in max_hashmap.into_iter() {
        match key {
            "abilities" => process_change(
                key,
                &val,
                min_hashmap.get(key).unwrap(),
                &mut change.abilities,
                sum,
            ),
            "items" => process_change(
                key,
                &val,
                min_hashmap.get(key).unwrap(),
                &mut change.items,
                sum,
            ),
            "runes" => process_change(
                key,
                &val,
                min_hashmap.get(key).unwrap(),
                &mut change.runes,
                sum,
            ),
            "spell" => process_change(
                key,
                &val,
                min_hashmap.get(key).unwrap(),
                &mut change.spell,
                sum,
            ),
            _ => continue,
        }
    }
    change
}

fn tool_change(max: &GamePlayerDamages, min: &GamePlayerDamages) -> TargetToolChange {
    let mut sum = 0.0;
    let dif = find_change(max, min, &mut sum);
    TargetToolChange { dif, sum }
}

fn tool_damage(
    mut active_player: GameActivePlayer,
    player: &GamePlayer,
    item: &str,
) -> GamePlayerTool {
    let assigned_stats = assing_stats(item, &mut active_player);
    active_player.champion_stats = GameChampionStats::from_hashmap_camel(assigned_stats);
    active_player.bonus_stats = Some(GameChampionStats::bonus_stats(
        &active_player.champion_stats,
        active_player.base_stats.unwrap(),
    ));

    let stats = all_stats(&player, &active_player);

    let damage_max = GamePlayerDamages {
        abilities: ability_damage(
            &stats,
            &active_player.abilities,
            &LOCAL_CHAMP.read().unwrap(),
        ),
        items: item_damage(
            &stats,
            &active_player.relevant.as_ref().unwrap().items.min,
            &LOCAL_ITEMS,
        ),
        runes: rune_damage(
            &stats,
            &active_player.relevant.as_ref().unwrap().runes.min,
            &LOCAL_RUNES,
        ),
        spell: spell_damage(
            &active_player.relevant.as_ref().unwrap().spell.min,
            active_player.level,
        ),
    };

    let change = tool_change(&damage_max, &player.damage.as_ref().unwrap());

    GamePlayerTool {
        sum: change.sum,
        dif: Some(change.dif),
        max: damage_max,
        rec: None,
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
        ("attackSpeed", 1.0),
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

    entries
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect()
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
                    rune.clone(),
                    GamePlayerDamage {
                        min,
                        max: None,
                        damage_type: val.rune_type.clone(),
                        name: Some(val.name.clone()),
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
                        Some(HashMap::from([("total".to_owned(), total.unwrap())]))
                    } else {
                        None
                    },
                );
                result.insert(
                    item.clone(),
                    GamePlayerDamage {
                        min,
                        max,
                        damage_type: val.item_type.clone(),
                        name: Some(val.name.clone()),
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
                spell.clone(),
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
            result.insert(key.clone(), GamePlayerDamage::void());
        }
        let min_str = &val.min[index];
        let max_str = val.max.as_ref().and_then(|t| t.get(index));

        let (min, max) = evaluate(min_str, max_str, stats, None);

        result.insert(
            key.clone(),
            GamePlayerDamage {
                min,
                max,
                damage_type: val.ability_type.clone(),
                name: None,
                area: None,
                onhit: None,
            },
        );
    }
    let acst = &stats.active_player.champion_stats;
    let attack = acst.attack_damage * stats.active_player.multiplier.physical;
    result.insert(
        "A".to_owned(),
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
        "C".to_owned(),
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

    if rel.runes.min.contains(&"8299".to_owned()) {
        if mshp > 0.7 {
            acp_mod += 0.11;
        } else if mshp >= 0.4 {
            acp_mod += 0.2 * mshp - 0.03;
        }
    }

    if rel.items.min.contains(&"4015".to_owned()) {
        acp_mod += exhp / (220000.0 / 15.0);
    }

    let form = if acs.attack_range > 350.0 {
        "ranged".to_owned()
    } else {
        "melee".to_owned()
    };

    let adaptative_type = if adp {
        "physical".to_owned()
    } else {
        "magic".to_owned()
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
