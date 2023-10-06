use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

use crate::db::dao::UserTransaction;
use crate::provider::helpers::get_explorer_url;

#[derive(Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_id: String,
    pub amount: Amount,
    pub metadata: Metadata,
    pub from: UserInfo,
    pub id: i32,
    pub to: UserInfo,
    #[serde(rename = "type")]
    pub transaction_type: String,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Amount {
    pub currency: String,
    pub value: BigDecimal,
    pub exponent: i32,
}

#[derive(Serialize, Deserialize)]
pub struct UserInfo {
    pub address: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub chain: String,
    pub gas: Amount,
    pub transaction_hash: String,
    pub timestamp: i64,
    pub explorer_url: String,
    pub status: String,
}

impl From<UserTransaction> for Transaction {
    fn from(transaction: UserTransaction) -> Self {
        Self {
            transaction_id: transaction.transaction_id,
            amount: Amount {
                currency: transaction.currency,
                value: transaction.amount,
                exponent: transaction.exponent,
            },
            metadata: Metadata {
                chain: transaction.metadata.chain,
                gas: Amount::default(),
                transaction_hash: transaction.metadata.transaction_hash.clone(),
                timestamp: transaction.updated_at.timestamp(),
                explorer_url: get_explorer_url(&transaction.metadata.transaction_hash),
                status: transaction.status,
            },
            from: UserInfo {
                address: transaction.from_address,
                name: transaction.metadata.from_name,
            },
            id: transaction.id,
            to: UserInfo {
                address: transaction.to_address,
                name: transaction.metadata.to_name,
            },
            transaction_type: transaction.transaction_type,
        }
    }
}
