use crate::CONFIG;
use bigdecimal::BigDecimal;
use log::error;
use sqlx::{query_as, Pool, Postgres};
use std::process::exit;

#[derive(Clone)]
pub struct Transactions {
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub value: Option<BigDecimal>,
    pub transaction_hash: Option<String>,
    pub block_number: Option<i64>,
    pub exponent: Option<i32>,
}

impl Transactions {
    pub fn get_max_block_number(transactions: Vec<Transactions>) -> i64 {
        transactions
            .into_iter()
            .max_by_key(|t| t.block_number)
            .unwrap()
            .block_number
            .unwrap_or(0)
    }

    pub async fn get(pool: Pool<Postgres>, block_number: i64) -> Vec<Transactions> {
        let query = query_as!(
            Transactions,
            "SELECT lower(t.from_address) from_address, lower(t.to_address) to_address, t.value, \
            lower(t.hash) transaction_hash, t.block_number, m.exponent \
            FROM transactions t \
            JOIN users u ON lower(t.to_address) = u.wallet_address \
            JOIN (SELECT exponent FROM token_metadata WHERE chain = $2 and contract_address = $3) m ON true \
            WHERE block_number > $1",
            block_number,
            CONFIG.get_chain(),
            "0x0000000000000000000000000000000000000000"
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
