use async_trait::async_trait;
use log::warn;
use sqlx::{Error, PgPool, Pool, Postgres};
use std::process::exit;

#[mockall::automock]
#[async_trait]
pub trait Connections {
    async fn init() -> Result<Pool<Postgres>, Error>;
}
pub struct DatabaseConnection;

#[async_trait]
impl Connections for DatabaseConnection {
    async fn init() -> Result<Pool<Postgres>, Error> {
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
    use sqlx::error::BoxDynError;

    #[tokio::test]
    async fn test_init_error() {
        let mock_pool = MockConnections::init_context();

        mock_pool.expect().returning(|| {
            Err(Error::Configuration(
                BoxDynError::try_from("DATABASE_URL not set").unwrap(),
            ))
        });

        let result = MockConnections::init().await;

        assert!(result.is_err())
    }
}
