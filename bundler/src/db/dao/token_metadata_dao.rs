use chrono::{DateTime, Utc};
use sqlx::{query, query_as, Error, Pool, Postgres};

use crate::errors::DatabaseError;

#[derive(Clone)]
pub struct TokenMetadataDao;

#[mockall::automock]
impl TokenMetadataDao {
    pub async fn add_metadata(
        pool: &Pool<Postgres>,
        chain: String,
        currency: String,
        address: String,
        exponent: i32,
        token_type: String,
        name: String,
        chain_id: i32,
        chain_name: String,
        token_image_url: String,
    ) -> Result<(), DatabaseError> {
        let query = query!(
            "INSERT INTO token_metadata \
            (chain, symbol, contract_address, exponent, token_type, name, chain_id, chain_name,\
             token_image_url) VALUES \
            ($1, $2, $3, $4, $5, $6, $7, $8, $9) on conflict (chain, symbol) do update set \
            contract_address = $3, exponent = $4, token_type = $5, name = $6, chain_id = $7, \
            chain_name = $8, token_image_url = $9, updated_at = now()",
            chain,
            currency,
            address,
            exponent,
            token_type,
            name,
            chain_id,
            chain_name,
            token_image_url
        );
        let result = query.execute(pool).await;
        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(DatabaseError::ServerError(format!(
                "Failed to create metadata: {}, err: {:?}",
                chain, err
            ))),
        }
    }

    pub async fn get_metadata_for_chain(
        pool: &Pool<Postgres>,
        chain: String,
        currency: Option<String>,
    ) -> Result<Vec<TokenMetadata>, DatabaseError> {
        let result: Result<Vec<TokenMetadata>, Error> = match currency {
            None => {
                let query = query_as!(
                    TokenMetadata,
                    "SELECT * FROM token_metadata WHERE lower(chain) = lower($1) and is_supported = true",
                    chain
                );
                query.fetch_all(pool).await
            }
            Some(currency) => {
                let query = query_as!(
                    TokenMetadata,
                    "SELECT * FROM token_metadata WHERE lower(chain) = lower($1) AND lower(symbol) = lower($2) and is_supported = true",
                    chain,
                    currency
                );
                query.fetch_all(pool).await
            }
        };
        match result {
            Ok(currencies) => Ok(currencies),
            Err(err) => Err(DatabaseError::ServerError(format!(
                "Failed to get currencies, err: {:?}",
                err
            ))),
        }
    }

    pub async fn get_metadata(pool: &Pool<Postgres>) -> Result<Vec<TokenMetadata>, DatabaseError> {
        let query = query_as!(
            TokenMetadata,
            "SELECT * FROM token_metadata where is_supported = true"
        );
        let result = query.fetch_all(pool).await;

        match result {
            Ok(metadata) => Ok(metadata),
            Err(err) => Err(DatabaseError::ServerError(format!(
                "Failed to get metadata, err: {:?}",
                err
            ))),
        }
    }
}

#[derive(Default, Clone)]
pub struct TokenMetadata {
    pub chain: String,
    pub symbol: String,
    pub contract_address: String,
    pub exponent: i32,
    pub token_type: String,
    pub name: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_supported: bool,
    pub chain_id: Option<i32>,
    pub chain_name: Option<String>,
    pub token_image_url: Option<String>,
}

#[cfg(test)]
mod test {
    use super::*;
    use sqlx::{Pool, Postgres};

    use crate::db::connection::MockDatabaseConnection;

    #[sqlx::test]
    async fn test_get_metadata_for_chain_no_records(pool: Pool<Postgres>) {
        let pool_context = MockDatabaseConnection::init_context();
        pool_context.expect().returning(move || Ok(pool.clone()));

        let mock_pool = MockDatabaseConnection::init().await.unwrap();
        let context = MockTokenMetadataDao::get_metadata_for_chain_context();

        context.expect().returning(|_, _, _| Ok(vec![]));

        let result =
            MockTokenMetadataDao::get_metadata_for_chain(&mock_pool, String::from("chain"), None)
                .await;

        assert!(result.is_ok());
    }

    #[sqlx::test]
    async fn test_get_metadata_for_chain_one_record(pool: Pool<Postgres>) {
        let pool_context = MockDatabaseConnection::init_context();
        pool_context.expect().returning(move || Ok(pool.clone()));

        let mock_pool = MockDatabaseConnection::init().await.unwrap();
        let context = MockTokenMetadataDao::get_metadata_for_chain_context();

        context
            .expect()
            .returning(|_, _, _| Ok(vec![TokenMetadata::default()]));

        let result =
            MockTokenMetadataDao::get_metadata_for_chain(&mock_pool, String::from("chain"), None)
                .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[sqlx::test]
    async fn test_get_metadata_for_chain_failure(pool: Pool<Postgres>) {
        let pool_context = MockDatabaseConnection::init_context();
        pool_context.expect().returning(move || Ok(pool.clone()));

        let mock_pool = MockDatabaseConnection::init().await.unwrap();
        let context = MockTokenMetadataDao::get_metadata_for_chain_context();

        context.expect().returning(|_, _, _| {
            Err(DatabaseError::ServerError(String::from(
                "Failed to get currencies",
            )))
        });

        let result =
            MockTokenMetadataDao::get_metadata_for_chain(&mock_pool, String::from("chain"), None)
                .await;

        assert!(result.is_err());
    }
}
