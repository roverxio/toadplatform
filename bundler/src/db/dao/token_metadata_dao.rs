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
            "INSERT INTO token_metadata (chain, symbol, contract_address, exponent, token_type, name) VALUES ($1, $2, $3, $4, $5, $6)",
            chain,
            currency,
            address,
            exponent,
            token_type,
            name);
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
        let result: Result<Vec<TokenMetadata>, Error> = match currency {
            None => {
                let query = query_as!(
                    TokenMetadata,
                    "SELECT * FROM token_metadata WHERE chain = $1 and is_supported = true",
                    chain
                );
                query.fetch_all(&self.pool).await
            }
            Some(currency) => {
                let query = query_as!(
                    TokenMetadata,
                    "SELECT * FROM token_metadata WHERE chain = $1 AND symbol = $2 and is_supported = true",
                    chain,
                    currency
                );
                query.fetch_all(&self.pool).await
            }
        };
        return match result {
            Ok(currencies) => currencies,
            Err(err) => {
                error!("Failed to get currencies, err: {:?}", err);
                vec![]
            }
        };
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
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub is_supported: bool,
}
