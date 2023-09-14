use log::error;
use sqlx::{PgPool, Pool, Postgres};
use std::process::exit;

pub struct Connection {}

impl Connection {
    pub async fn init() -> Pool<Postgres> {
        match std::env::var("DATABASE_URL") {
            Ok(database_url) => {
                let connection = PgPool::connect(&database_url).await;
                match connection {
                    Ok(conn) => conn,
                    Err(_) => {
                        error!("Unable to open connection to the database");
                        exit(1)
                    }
                }
            }
            Err(_) => {
                error!("env DATABASE_URL variable not set");
                exit(1)
            }
        }
    }
}
