use log::warn;
use r2d2::Pool;
use r2d2_sqlite::rusqlite::params_from_iter;
use r2d2_sqlite::SqliteConnectionManager;
use sqlx::{query, query_as, Error, Postgres};

use crate::db::dao::connect::connect;

#[derive(Clone)]
pub struct MetadataDao {
    pub pool: Pool<SqliteConnectionManager>,
    pub db_pool: sqlx::Pool<Postgres>,
}

impl MetadataDao {
    pub async fn add_metadata(
        &self,
        chain: String,
        currency: String,
        address: String,
        exponent: i32,
    ) {
        let query = query!(
            "INSERT INTO supported_currencies (chain, currency, contract_address, exponent) VALUES ($1, $2, $3, $4)",
            chain,
            currency,
            address,
            exponent);
        let result = query.execute(&self.db_pool).await;
        if result.is_err() {
            warn!("Failed to create metadata: {}", chain);
        }
    }

    pub async fn get_metadata_for_chain(
        &self,
        chain: String,
        currency: Option<String>,
    ) -> Vec<SupportedCurrency> {
        let result: Result<Vec<SupportedCurrency>, Error> = match currency {
            None => {
                let query = query_as!(
                    SupportedCurrency,
                    "SELECT chain, currency, exponent FROM supported_currencies WHERE chain = $1",
                    chain
                );
                query.fetch_all(&self.db_pool).await
            }
            Some(currency) => {
                let query = query_as!(
                SupportedCurrency,
                "SELECT chain, currency, exponent FROM supported_currencies WHERE chain = $1 AND currency = $2",
                chain,
                currency
            );
                query.fetch_all(&self.db_pool).await
            }
        };
        return match result {
            Ok(currencies) => currencies,
            Err(_) => vec![],
        };
    }
}

#[derive(Default, Clone)]
pub struct SupportedCurrency {
    pub chain: String,
    pub currency: String,
    pub exponent: i32,
}
