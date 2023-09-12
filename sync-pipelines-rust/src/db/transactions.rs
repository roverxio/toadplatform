use bigdecimal::BigDecimal;
use log::error;
use sqlx::{query_as, Pool, Postgres};
use std::process::exit;

#[derive(Clone)]
pub struct Transactions {
    pub from_address: String,
    pub to_address: String,
    pub value: BigDecimal,
    pub transaction_hash: String,
    pub block_timestamp: i64,
}

impl Transactions {
    pub fn get_max_block_timestamp(transactions: Vec<Transactions>) -> i64 {
        transactions
            .into_iter()
            .max_by_key(|t| t.block_timestamp)
            .unwrap()
            .block_timestamp
    }
}

impl Transactions {
    pub async fn get(pool: Pool<Postgres>, block_timestamp: i64) -> Vec<Transactions> {
        let query = query_as!(
            Transactions,
            "SELECT from_address, to_address, value, hash as transaction_hash, block_timestamp \
            FROM transactions \
            JOIN users ON to_address = wallet_address \
            WHERE block_timestamp > $1",
            block_timestamp
        );
        let result = query.fetch_all(&pool).await;
        return match result {
            Ok(rows) => rows,
            Err(error) => {
                error!("Failed to fetch transactions: {:?}", error);
                exit(1);
            }
        };
    }
}
