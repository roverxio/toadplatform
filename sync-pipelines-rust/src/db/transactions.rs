use crate::utils::utils::Utils;
use bigdecimal::BigDecimal;
use log::error;
use sqlx::{query_as, Pool, Postgres};

pub struct Transactions {
    pub from_address: String,
    pub to_address: String,
    pub value: BigDecimal,
    pub transaction_hash: String,
    pub block_timestamp: i64,
}

impl Transactions {
    pub async fn get(pool: Pool<Postgres>) -> Vec<Transactions> {
        let start_time = Utils::get_last_synced_time();
        let query = query_as!(
            Transactions,
            "SELECT from_address, to_address, value, hash as transaction_hash, block_timestamp \
            FROM transactions \
            JOIN users ON to_address = wallet_address \
            WHERE block_timestamp > $1",
            start_time
        );
        let result = query.fetch_all(&pool).await;
        return match result {
            Ok(rows) => rows,
            Err(error) => {
                error!("Failed to fetch transactions: {:?}", error);
                vec![]
            }
        };
    }
}
