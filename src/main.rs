use actix_cors::Cors;
use std::env;
use std::fs::File as FileSync;
use std::io::BufReader;

use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use sea_orm::{Database, DatabaseConnection};
use serde::de::DeserializeOwned;
use std::error::Error;
use tokio::fs::File;

use tokio::io::AsyncReadExt;

mod entity;
mod routes;
mod services;
mod structs;

use serde::Serialize;

pub fn structured_clone<T>(value: &T) -> T
where
    T: Serialize + DeserializeOwned,
{
    serde_json::from_str(&serde_json::to_string(value).unwrap()).unwrap()
}

pub fn fetch_json_sync<T>(path: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T: DeserializeOwned,
{
    let file = FileSync::open(format!("{}.json", path))?;
    let reader = BufReader::new(file);
    let data = serde_json::from_reader(reader)?;
    Ok(data)
}

pub async fn fetch_json<T>(path: &str) -> Result<T, Box<dyn Error>>
where
    T: DeserializeOwned,
{
    let mut file = File::open(format!("{}.json", path)).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    let data = serde_json::from_str(&contents)?;
    Ok(data)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db: DatabaseConnection = Database::connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:5173")
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![
                        actix_web::http::header::AUTHORIZATION,
                        actix_web::http::header::ACCEPT,
                    ])
                    .allowed_header(actix_web::http::header::CONTENT_TYPE)
                    .max_age(3600),
            )
            .configure(routes::index::config)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
