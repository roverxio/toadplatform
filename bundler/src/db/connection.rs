use log::warn;
use sqlx::{Error, PgPool, Pool, Postgres};
use std::process::exit;

pub struct DatabaseConnection;

#[mockall::automock]
impl DatabaseConnection {
    pub async fn init() -> Result<Pool<Postgres>, Error> {
        let database_url: String;
        match std::env::var("DATABASE_URL") {
            Ok(url) => database_url = url,
            Err(_) => {
                warn!("env DATABASE_URL not set");
                exit(1)
            }
        }
        PgPool::connect(&database_url).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Pool, Postgres};

    #[sqlx::test]
    async fn test_init_success(pool: Pool<Postgres>) {
        let mock_pool = MockDatabaseConnection::init_context();

        mock_pool.expect().returning(move || Ok(pool.clone()));

        let result = MockDatabaseConnection::init().await;

        assert!(result.is_ok());
        mock_pool.checkpoint();
    }

    #[tokio::test]
    async fn test_init_failure() {
        let mock_pool = MockDatabaseConnection::init_context();

        mock_pool.expect().returning(|| Err(Error::PoolClosed));

        let result = MockDatabaseConnection::init().await;

        assert!(result.is_err());
        mock_pool.checkpoint();
    }
}
