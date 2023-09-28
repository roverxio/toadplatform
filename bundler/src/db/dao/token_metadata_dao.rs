use crate::errors::base::DatabaseError;
use chrono::{DateTime, Utc};
use log::error;
use sqlx::{query, query_as, Error, Pool, Postgres};

#[derive(Clone)]
pub struct TokenMetadataDao {
    pub pool: Pool<Postgres>,
}

impl TokenMetadataDao {
    pub async fn add_metadata(
        &self,
        chain: String,
        currency: String,
        address: String,
        exponent: i32,
        token_type: String,
        name: String,
        chain_id: i32,
        chain_name: String,
        token_image_url: String,
    ) {
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
        let result = query.execute(&self.pool).await;
        if result.is_err() {
            error!(
                "Failed to create metadata: {}, err: {:?}",
                chain,
                result.err()
            );
        }
    }

    pub async fn get_metadata_for_chain(
        &self,
        chain: String,
        currency: Option<String>,
    ) -> Vec<TokenMetadata> {
        Self::get_metadata_by_currency(&self.pool, chain, currency)
            .await
            .unwrap()
    }

    pub async fn get_metadata_by_currency(
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
            Err(err) => {
                error!("Failed to get currencies, err: {:?}", err);
                Err(DatabaseError(String::from("Failed to get currencies")))
            }
        }
    }

    pub async fn get_metadata(&self) -> Vec<TokenMetadata> {
        let query = query_as!(
            TokenMetadata,
            "SELECT * FROM token_metadata where is_supported = true"
        );
        let result = query.fetch_all(&self.pool).await;

        match result {
            Ok(metadata) => metadata,
            Err(err) => {
                error!("Failed to get metadata, err: {:?}", err);
                vec![]
            }
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
mod tests {
    use crate::db::connection::DatabaseConnection;
    use crate::db::dao::token_metadata_dao::TokenMetadataDao;
    use crate::CONFIG;

    #[sqlx::test]
    async fn test_get_metadata_without_currency() {
        let pool = DatabaseConnection::init().await;
        let chain = CONFIG.run_config.current_chain.clone();
        let result = TokenMetadataDao::get_metadata_by_currency(&pool, chain, None).await;
        assert!(result.is_ok());
    }

    #[sqlx::test]
    async fn test_get_metadata_with_currency() {
        let pool = DatabaseConnection::init().await;
        let chain = CONFIG.run_config.current_chain.clone();
        let result =
            TokenMetadataDao::get_metadata_by_currency(&pool, chain, Some(String::from("ETH")))
                .await;
        assert!(result.is_ok());
    }
}
