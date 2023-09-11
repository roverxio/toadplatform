use crate::utils::utils::Utils;
use bigdecimal::BigDecimal;
use log::error;
use sqlx::{query_as, Pool, Postgres};

pub struct TokenTransfers {
    pub from_address: String,
    pub to_address: String,
    pub value: BigDecimal,
    pub transaction_hash: String,
    pub block_number: i64,
    pub symbol: String,
}

impl TokenTransfers {
    pub async fn get(pool: Pool<Postgres>) -> Vec<TokenTransfers> {
        let block_number = Utils::get_last_synced_block_number();
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
                vec![]
            }
        };
    }
}
