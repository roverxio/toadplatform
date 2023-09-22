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
    ) {
        let query = query!(
            "INSERT INTO token_metadata \
            (chain, symbol, contract_address, exponent, token_type, name) VALUES \
            ($1, $2, $3, $4, $5, $6) on conflict (chain, symbol) do update set \
            contract_address = $3, exponent = $4, token_type = $5, name = $6, updated_at = now()",
            chain,
            currency,
            address,
            exponent,
            token_type,
            name
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
        Self::get_metadata(&self.pool, chain, currency)
            .await
            .unwrap()
    }

    pub async fn get_metadata(
        pool: &Pool<Postgres>,
        chain: String,
        currency: Option<String>,
    ) -> Result<Vec<TokenMetadata>, String> {
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
                Err(String::from("Failed to get currencies"))
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
}
