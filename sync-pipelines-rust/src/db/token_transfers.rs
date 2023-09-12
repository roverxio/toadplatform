use bigdecimal::BigDecimal;
use log::error;
use sqlx::{query_as, Pool, Postgres};
use std::process::exit;

#[derive(Clone)]
pub struct TokenTransfers {
    pub from_address: String,
    pub to_address: String,
    pub value: BigDecimal,
    pub transaction_hash: String,
    pub block_number: i64,
    pub symbol: String,
}

impl TokenTransfers {
    pub fn get_max_block_number(transfers: Vec<TokenTransfers>) -> i64 {
        transfers
            .into_iter()
            .max_by_key(|t| t.block_number)
            .unwrap()
            .block_number
    }

    pub async fn get(pool: Pool<Postgres>, block_number: i64) -> Vec<TokenTransfers> {
        let query = query_as!(
            TokenTransfers,
            "SELECT from_address, to_address, value, transaction_hash, block_number, symbol \
            FROM token_transfers \
            JOIN users ON to_address = wallet_address \
            JOIN token_metadata ON token_address = contract_address \
            WHERE block_number > $1",
            block_number
        );
        let result = query.fetch_all(&pool).await;
        return match result {
            Ok(rows) => rows,
            Err(error) => {
                error!("Failed to fetch token_transfers: {:?}", error);
                exit(1);
            }
        };
    }
}
