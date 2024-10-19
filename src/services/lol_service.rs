use std::collections::HashMap;

use crate::{
    fetch_json,
    structs::{
        riot_champion_struct::RiotChampion,
        riot_items_struct::RiotItems,
        target_struct::{RiotChampionTarget, RiotChampionTargetSpell, RiotItemTarget},
    },
};

async fn get_champion(champion: &str) -> String {
    let t = fetch_json::<HashMap<String, HashMap<String, String>>>("src/cache/ids")
        .await
        .unwrap();
    for (key, val) in t.into_iter() {
        for (_, v) in val.into_iter() {
            if key == v {
                println!("Found: {}", key);
                return key;
            }
        }
    }
    println!("Did not find: {}", champion);
    champion.to_string()
}

pub async fn item_api(item: &str) -> Result<RiotItemTarget, Box<dyn std::error::Error>> {
    let items = fetch_json::<RiotItems>("src/cache/item").await?;
    let t = items.data[item].clone();
    Ok(RiotItemTarget {
        name: t.name.unwrap(),
        description: t.description.unwrap(),
        stats: t.stats,
        gold: t.gold.unwrap(),
        maps: t.maps.unwrap(),
        from: t.from,
    })
}

pub async fn champion_api(
    champion: &str,
) -> Result<RiotChampionTarget, Box<dyn std::error::Error>> {
    let name = get_champion(champion).await;
    let x = fetch_json::<RiotChampion>(&format!("src/cache/champions/{}", &name)).await?;
    let t = x.data[&name].clone();
    Ok(RiotChampionTarget {
        id: t.id,
        name: t.name,
        stats: t.stats,
        spells: t
            .spells
            .iter()
            .map(|z| RiotChampionTargetSpell {
                id: z.id.clone(),
                name: z.name.clone(),
                description: z.description.clone(),
                cooldown: z.cooldown.clone(),
            })
            .collect(),
        passive: t.passive,
    })
}
