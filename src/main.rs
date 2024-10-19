use serde::de::DeserializeOwned;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use services::game_service::calculate;

mod services;
mod structs;

use structs::game_struct::GameProps;

pub async fn fetch_json<T>(path: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T: DeserializeOwned,
{
    // println!("Fetching: {}", path);
    let mut file = File::open(format!("{}.json", path)).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    let data = serde_json::from_str(&contents)?;
    // println!("Sucess");
    Ok(data)
}

#[tokio::main(flavor = "multi_thread", worker_threads = 6)]
// #[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = fetch_json::<GameProps>("test").await?;

    calculate(data).await;

    Ok(())
}
