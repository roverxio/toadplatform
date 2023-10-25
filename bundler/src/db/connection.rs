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
    use mockall::mock;
    use sqlx::{Pool, Postgres};
    use std::future::Future;

    mock! {
        DatabaseConnection {
            fn init() -> impl Future<Output = Result<Pool<Postgres>, Error>>;
        }
    }

    #[tokio::test]
    async fn test_init_success() {
        let mock_pool = MockDatabaseConnection::init_context();

        mock_pool.expect().returning(|| {
            Box::pin(async {
                let db_url = std::env::var("DATABASE_URL").unwrap();
                let pool = PgPool::connect(&db_url).await;
                Ok(pool.unwrap())
            })
        });

        let result = MockDatabaseConnection::init().await;

        assert!(result.is_ok());
        mock_pool.checkpoint();
    }

    #[tokio::test]
    async fn test_init_failure() {
        let mock_pool = MockDatabaseConnection::init_context();

        mock_pool
            .expect()
            .returning(|| Box::pin(async { Err(Error::PoolClosed) }));

        let result = MockDatabaseConnection::init().await;

        assert!(result.is_err());
        mock_pool.checkpoint();
    }
}
