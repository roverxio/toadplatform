use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::db::dao::connect::connect;

#[derive(Clone)]
pub struct MetadataDao {
    pub pool: Pool<SqliteConnectionManager>,
}

impl MetadataDao {
    pub async fn add_metadata(
        &self,
        chain: String,
        currency: String,
        address: String,
        exponent: u8,
    ) {
        let conn = connect(self.pool.clone()).await;
        let mut stmt = conn
            .prepare(
                "INSERT OR IGNORE INTO supported_currencies (chain, currency, contract_address, exponent) VALUES (?1, ?2, ?3, ?4)",
            )
            .unwrap();
        stmt.execute([chain, currency, address, exponent.to_string()])
            .unwrap();
    }

    pub async fn get_metadata_for_chain(&self, chain: String) -> Vec<SupportedCurrency> {
        let conn = connect(self.pool.clone()).await;
        let mut stmt = conn
            .prepare("SELECT chain, currency, exponent FROM supported_currencies WHERE chain = ?1")
            .unwrap();

        let rows: Vec<SupportedCurrency> = stmt
            .query_map([chain], |row| {
                Ok(SupportedCurrency {
                    chain: row.get(0)?,
                    currency: row.get(1)?,
                    exponent: row.get(2)?,
                })
            })
            .and_then(Iterator::collect)
            .unwrap();
        rows
    }
}

#[derive(Clone)]
pub struct SupportedCurrency {
    pub chain: String,
    pub currency: String,
    pub exponent: u8,
}
