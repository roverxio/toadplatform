use sqlx::{PgPool, Pool, Postgres};

pub struct Connection {}

impl Connection {
    pub async fn init() -> Result<Pool<Postgres>, String> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| String::from("env DATABASE_URL variable not set"))?;
        let connection = PgPool::connect(&database_url)
            .await
            .map_err(|_| String::from("Failed to connect to database"))?;
        Ok(connection)
    }
}
