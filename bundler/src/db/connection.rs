use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use sqlx::{PgPool, Postgres};

pub fn establish_connection(database_url: String) -> Pool<SqliteConnectionManager> {
    let manager = SqliteConnectionManager::file(&database_url);
    Pool::new(manager).unwrap()
}

pub struct DatabaseConnection {}

impl DatabaseConnection {
    pub async fn init(database_url: String) -> sqlx::Pool<Postgres> {
        let connection = PgPool::connect(&database_url).await;
        connection.unwrap()
    }
}
