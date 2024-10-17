use std::fs::File;
use std::io::BufReader;

use services::game_service::calculate;

mod services;
mod structs;

use structs::game_struct::GameProps;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("test.json")?;
    let reader = BufReader::new(file);

    let data: GameProps = serde_json::from_reader(reader)?;

    println!("{:#?}", data);

    let x = calculate();
    println!("{}", x);

    Ok(())
}
