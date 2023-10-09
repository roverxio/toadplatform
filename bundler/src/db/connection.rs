use sqlx::{PgPool, Pool, Postgres};

pub struct DatabaseConnection;

impl DatabaseConnection {
    pub async fn init() -> Result<Pool<Postgres>, String> {
        let database_url: String;
        match std::env::var("DATABASE_URL") {
            Ok(url) => database_url = url,
            Err(_) => return Err(String::from("env DATABASE_URL not set")),
        }
        let connection = PgPool::connect(&database_url).await;
        match connection {
            Ok(db_pool) => Ok(db_pool),
            Err(err) => Err(format!("Failed to connect to db: {:?}", err)),
        }
    }
}
