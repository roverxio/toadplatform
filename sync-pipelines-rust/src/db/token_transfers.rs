use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};

pub struct TokenTransfers {
    pub token_address: String,
    pub from_address: String,
    pub to_address: String,
    pub value: BigDecimal,
    pub transaction_hash: String,
    pub block_timestamp: DateTime<Utc>,
}

impl TokenTransfers {
    pub fn get(_start_time: DateTime<Utc>) -> Vec<TokenTransfers> {
        unimplemented!();
    }
}
