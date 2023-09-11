use log::warn;
use sqlx::{PgPool, Pool, Postgres};
use std::process::exit;

pub struct Connection {}

impl Connection {
    pub async fn init() -> Pool<Postgres> {
        let database_url: String;
        match std::env::var("DATABASE_URL") {
            Ok(url) => database_url = url,
            Err(_) => {
                warn!("env DATABASE_URL not set");
                exit(1)
            }
        }
        let connection = PgPool::connect(&database_url).await;
        connection.unwrap()
    }
}
