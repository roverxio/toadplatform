use bigdecimal::num_bigint::BigInt;
use bigdecimal::BigDecimal;
use sqlx::{Pool, Postgres};

pub struct TokenTransfers {
    pub token_address: String,
    pub from_address: String,
    pub to_address: String,
    pub value: BigDecimal,
    pub transaction_hash: String,
    pub block_number: u64,
}

impl TokenTransfers {
    pub fn get(_pool: Pool<Postgres>) -> Vec<TokenTransfers> {
        unimplemented!()
    }
}
