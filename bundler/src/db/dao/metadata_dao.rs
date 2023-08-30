use log::warn;
use r2d2::Pool;
use r2d2_sqlite::rusqlite::params_from_iter;
use r2d2_sqlite::SqliteConnectionManager;
use sqlx::{query, Postgres};

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
        let conn = connect(self.pool.clone()).await;
        let mut query =
            "SELECT chain, currency, exponent FROM supported_currencies WHERE chain = ?1"
                .to_string();
        let mut values = vec![chain];
        match currency {
            None => {}
            Some(currency) => {
                query = format!("{} AND currency = ?2", query);
                values.push(currency);
            }
        }
        let mut stmt = conn.prepare(query.as_str()).unwrap();

        let rows: Vec<SupportedCurrency> = stmt
            .query_map(params_from_iter(values), |row| {
                Ok(SupportedCurrency {
                    chain: row.get(0)?,
                    currency: row.get(1)?,
                    exponent: row.get(2)?,
                })
            })
            .and_then(Iterator::collect)
            .unwrap();
        if rows.len() > 0 {
            rows
        } else {
            vec![SupportedCurrency::default()]
        }
    }
}

#[derive(Default, Clone)]
pub struct SupportedCurrency {
    pub chain: String,
    pub currency: String,
    pub exponent: i32,
}
