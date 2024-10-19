use std::sync::Arc;

use futures::stream::FuturesUnordered;
use futures::stream::StreamExt;

use super::lol_service::{champion_api, item_api};
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

async fn assign_champion(g: &mut GameProps) {
    for player in &mut g.all_players.iter_mut() {
        if player.summoner_name == g.active_player.summoner_name {
            g.active_player.team = Some(player.team.clone());
        }
        let c = champion_api(&player.champion_name).await;
        match c {
            Ok(c) => player.champion = Some(c),
            Err(e) => panic!("{:?}", e),
        }
    }
}

pub async fn calculate(mut data: GameProps) -> () {
    let local_runes = fetch_json::<LocalRunes>("src/effects/runes").await.unwrap();
    let local_items = fetch_json::<LocalItems>("src/effects/items").await.unwrap();
    let mut local_champ: LocalChampion;

    assign_champion(&mut data).await;

    let mut active_player = Arc::new(data.active_player);
    let mut all_players = data.all_players;

    for player in all_players.iter() {
        if player.summoner_name == active_player.summoner_name {
            if let Some(champion) = &player.champion {
                local_champ =
                    fetch_json::<LocalChampion>(&format!("src/champions/{}", &champion.id))
                        .await
                        .unwrap();

                let acp = Arc::make_mut(&mut active_player);

                acp.champion_name = Some(champion.name.clone());
                acp.skin = Some(player.skin_id);

                acp.base_stats = Some(GameCoreStats::base_stats(&champion.stats, player.level));
                acp.bonus_stats = Some(GameChampionStats::bonus_stats(
                    &acp.champion_stats,
                    acp.base_stats.unwrap(),
                ));

                acp.items = Some(
                    player
                        .items
                        .iter()
                        .map(|item| item.item_id.to_string())
                        .collect(),
                );

                acp.relevant = Some(GameRelevant {
                    abilities: filter_abilities(&local_champ, &champion.id),
                    items: filter_items(&local_items, &acp.items.clone().unwrap()),
                    runes: filter_runes(&local_runes, &acp.full_runes),
                    spell: filter_spell(&player.summoner_spells),
                });
            }
        }
    }

    let mut futures = FuturesUnordered::new();

    let acp_team = active_player.team.clone().unwrap();

    for player in all_players.iter_mut() {
        let active_player_clone = Arc::clone(&active_player);
        if player.team != acp_team {
            let future = async move {
                player.base_stats = Some(GameCoreStats::base_stats(
                    &player.champion.as_ref().unwrap().stats,
                    player.level,
                ));

                let items: Vec<String> = player
                    .items
                    .iter()
                    .map(|item| item.item_id.to_string())
                    .collect();

                player.champion_stats = Some(player_stats(player.base_stats.unwrap(), items).await);

                player.bonus_stats = Some(GameCoreStats::bonus_stats(
                    &player.base_stats.unwrap(),
                    &player.champion_stats.unwrap(),
                ));

                let stats = all_stats(&player, &active_player_clone);
            };
            futures.push(future);
        }
    }

    while let Some(t) = futures.next().await {
        println!("{:#?}", t);
    }
}

async fn all_stats(player: &GamePlayer, active_player: &GameActivePlayer) -> TargetAllStats {
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

fn filter_abilities(_champion: &LocalChampion, id: &str) -> GameRelevantProps {
    let mut min = Vec::with_capacity(8);
    let mut max = Vec::with_capacity(8);
    println!("ID: {}", id);
    for (key, val) in _champion[id].iter() {
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
