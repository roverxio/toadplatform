use bigdecimal::BigDecimal;
use sqlx::{query_as, Pool, Postgres};

#[derive(Clone)]
pub struct TokenTransfers {
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub value: Option<BigDecimal>,
    pub transaction_hash: Option<String>,
    pub block_number: Option<i64>,
    pub symbol: Option<String>,
    pub exponent: Option<i32>,
}

impl TokenTransfers {
    pub fn get_max_block_number(transfers: Vec<TokenTransfers>) -> i64 {
        transfers
            .into_iter()
            .max_by_key(|t| t.block_number)
            .unwrap()
            .block_number
            .unwrap_or(0)
    }

    pub async fn get(
        pool: Pool<Postgres>,
        block_number: i64,
    ) -> Result<Vec<TokenTransfers>, String> {
        let query = query_as!(
            TokenTransfers,
            "SELECT t.from_address, t.to_address, t.value, \
            t.transaction_hash, t.block_number, lower(m.symbol) symbol, m.exponent \
            FROM token_transfers t \
            JOIN users u ON t.to_address = u.wallet_address \
            JOIN token_metadata m ON t.token_address = m.contract_address \
            WHERE block_number > $1",
            block_number
        );
        let result = query.fetch_all(&pool).await;
        return match result {
            Ok(rows) => Ok(rows),
            Err(error) => Err(format!("Failed to fetch token_transfers: {:?}", error)),
        };
    }
}
