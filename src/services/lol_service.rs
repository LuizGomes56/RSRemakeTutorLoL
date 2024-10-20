use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

use crate::{
    fetch_json, fetch_json_sync,
    structs::{
        riot_champion_struct::RiotChampion,
        riot_items_struct::RiotItems,
        target_struct::{RiotChampionTarget, RiotChampionTargetSpell, RiotItemTarget},
    },
};

static ITEM_CACHE: Lazy<RiotItems> = Lazy::new(|| {
    fetch_json_sync::<RiotItems>("src/cache/item").expect("Erro ao carregar o arquivo de itens.")
});

static IDS_CACHE: Lazy<HashMap<String, HashMap<String, String>>> = Lazy::new(|| {
    fetch_json_sync::<HashMap<String, HashMap<String, String>>>("src/cache/ids")
        .expect("Erro ao carregar o arquivo de IDS.")
});

static CHAMPION_CACHE: Lazy<RwLock<HashMap<String, RiotChampionTarget>>> =
    Lazy::new(|| RwLock::new(HashMap::with_capacity(10)));

async fn get_champion(champion: &str) -> String {
    for (key, val) in IDS_CACHE.iter() {
        for (_, v) in val.iter() {
            if v == champion {
                return key.clone();
            }
        }
    }
    String::from("TargetDummy")
}

pub async fn item_api(item: &str) -> Result<RiotItemTarget, Box<dyn std::error::Error>> {
    let t = ITEM_CACHE
        .data
        .get(item)
        .ok_or("Item não encontrado")?
        .clone();
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

    {
        let cache = CHAMPION_CACHE.read().unwrap();
        if let Some(champ) = cache.get(&name) {
            return Ok(RiotChampionTarget {
                id: champ.id.clone(),
                name: champ.name.clone(),
                stats: champ.stats,
                spells: champ
                    .spells
                    .iter()
                    .map(|z| RiotChampionTargetSpell {
                        id: z.id.clone(),
                        name: z.name.clone(),
                        description: z.description.clone(),
                        cooldown: z.cooldown.clone(),
                    })
                    .collect(),
                passive: champ.passive.clone(),
            });
        }
    }

    let x = fetch_json::<RiotChampion>(&format!("src/cache/champions/{}", &name)).await?;
    let t = &x.data[&name];

    let result = RiotChampionTarget {
        id: t.id.clone(),
        name: t.name.clone(),
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
        passive: t.passive.clone(),
    };

    {
        let mut cache = CHAMPION_CACHE.write().unwrap();
        cache.insert(name.clone(), result.clone());
    }

    Ok(result)
}
