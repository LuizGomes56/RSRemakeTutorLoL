use std::fs::File as FileSync;
use std::io::BufReader;
use std::time::Instant;

use serde::de::DeserializeOwned;
use tokio::fs::File;

use tokio::io::AsyncReadExt;

use services::game_service::calculate;

mod services;
mod structs;

use structs::game_struct::GameProps;

pub fn fetch_json_sync<T>(path: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T: DeserializeOwned,
{
    println!("SYNC: {}", path);
    let file = FileSync::open(format!("{}.json", path))?;
    let reader = BufReader::new(file);
    let data = serde_json::from_reader(reader)?;
    Ok(data)
}

pub async fn fetch_json<T>(path: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T: DeserializeOwned,
{
    println!("ASYNC: {}", path);
    let mut file = File::open(format!("{}.json", path)).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    let data = serde_json::from_str(&contents)?;
    Ok(data)
}

#[tokio::main(flavor = "multi_thread", worker_threads = 6)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let now = Instant::now();

    let data = fetch_json::<GameProps>("test").await?;

    for _i in 0..5 {
        calculate(data.clone()).await;
    }

    let elapsed = now.elapsed();
    println!("Elapsed: {:?}", elapsed);
    Ok(())
}
