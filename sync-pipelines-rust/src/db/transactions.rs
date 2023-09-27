use crate::CONFIG;
use bigdecimal::BigDecimal;
use sqlx::{query_as, Pool, Postgres};

#[derive(Clone)]
pub struct Transactions {
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub value: Option<BigDecimal>,
    pub transaction_hash: String,
    pub block_number: Option<i64>,
}

impl Transactions {
    pub async fn get(pool: Pool<Postgres>, block_number: i64) -> Result<Vec<Transactions>, String> {
        let query = query_as!(
            Transactions,
            "SELECT t.from_address, t.to_address, t.value, t.hash transaction_hash, t.block_number \
            FROM transactions t JOIN users u ON t.to_address = u.wallet_address \
            JOIN (SELECT exponent FROM token_metadata WHERE chain = $2 and lower(token_type)='native') m ON true \
            WHERE block_number > $1 ORDER BY block_number DESC",
            block_number,
            CONFIG.get_chain(),
        );
        let result = query.fetch_all(&pool).await;
        return match result {
            Ok(rows) => Ok(rows),
            Err(error) => Err(format!("Failed to fetch transactions: {:?}", error)),
        };
    }
}
