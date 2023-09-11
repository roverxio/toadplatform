use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};

pub struct Transactions {
    pub from_address: String,
    pub to_address: String,
    pub value: BigDecimal,
    pub transaction_hash: String,
    pub block_timestamp: DateTime<Utc>,
}

impl Transactions {
    pub fn get(_pool: Pool<Postgres>) -> Vec<Transactions> {
        unimplemented!()
    }
}
