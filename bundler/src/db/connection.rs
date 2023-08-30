use sqlx::{PgPool, Postgres};

pub struct DatabaseConnection {}

impl DatabaseConnection {
    pub async fn init(database_url: String) -> sqlx::Pool<Postgres> {
        let connection = PgPool::connect(&database_url).await;
        connection.unwrap()
    }
}
