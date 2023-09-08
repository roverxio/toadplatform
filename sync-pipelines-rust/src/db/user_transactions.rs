use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

use crate::db::token_transfers::TokenTransfers;
use crate::db::transactions::Transactions;

#[derive(Clone, Default)]
pub struct UserTransaction {
    pub user_address: String,
    pub transaction_id: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: BigDecimal,
    pub currency: String,
    pub transaction_type: String,
    pub status: String,
    pub metadata: TransactionMetadata,
    pub exponent: i32,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct TransactionMetadata {
    pub chain: String,
    pub to_name: String,
    pub gas_erc20: Gas,
    pub gas: Gas,
    pub from_name: String,
    pub transaction_hash: String,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Gas {
    pub currency: String,
    pub value: u64,
}

impl From<TokenTransfers> for UserTransaction {
    fn from(_transfers: TokenTransfers) -> UserTransaction {
        unimplemented!()
    }
}

impl From<Transactions> for UserTransaction {
    fn from(_transfers: Transactions) -> UserTransaction {
        unimplemented!()
    }
}

impl UserTransaction {
    pub fn insert(_transactions: Vec<UserTransaction>) {
        unimplemented!();
    }

    pub fn from_token_transfers(_transfers: Vec<TokenTransfers>) -> Vec<UserTransaction> {
        unimplemented!()
    }

    pub fn from_transactions(_transfers: Vec<Transactions>) -> Vec<UserTransaction> {
        unimplemented!()
    }
}
