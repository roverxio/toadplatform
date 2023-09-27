use bigdecimal::BigDecimal;
use sqlx::{query_as, Pool, Postgres};

#[derive(Clone)]
pub struct TokenTransfers {
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub value: Option<BigDecimal>,
    pub transaction_hash: String,
    pub block_number: Option<i64>,
    pub symbol: Option<String>,
}

impl TokenTransfers {
    pub async fn get(
        pool: Pool<Postgres>,
        block_number: i64,
    ) -> Result<Vec<TokenTransfers>, String> {
        let query = query_as!(
            TokenTransfers,
            "SELECT t.from_address, t.to_address, t.value, \
            t.transaction_hash, t.block_number, lower(m.symbol) symbol \
            FROM token_transfers t \
            JOIN users u ON t.to_address = u.wallet_address \
            JOIN token_metadata m ON t.token_address = m.contract_address \
            WHERE block_number > $1 ORDER BY block_number desc",
            block_number
        );
        let result = query.fetch_all(&pool).await;
        return match result {
            Ok(rows) => Ok(rows),
            Err(error) => Err(format!("Failed to fetch token_transfers: {:?}", error)),
        };
    }
}
