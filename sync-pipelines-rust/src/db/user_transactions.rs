use crate::CONFIG;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

use crate::db::token_transfers::TokenTransfers;
use crate::db::transactions::Transactions;
use crate::utils::utils::Utils;

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

impl TransactionMetadata {
    pub fn get_transaction_metadata(transaction_hash: String) -> TransactionMetadata {
        TransactionMetadata {
            transaction_hash,
            chain: CONFIG.get_chain().to_string(),
            gas_erc20: Gas {
                value: 0,
                currency: "".to_string(),
            },
            gas: Gas {
                value: 0,
                currency: "".to_string(),
            },
            from_name: "".to_string(),
            to_name: "".to_string(),
        }
    }
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Gas {
    pub currency: String,
    pub value: u64,
}

impl From<TokenTransfers> for UserTransaction {
    fn from(transfer: TokenTransfers) -> UserTransaction {
        UserTransaction {
            user_address: transfer.to_address.clone(),
            transaction_id: Utils::generate_txn_id(),
            from_address: transfer.from_address,
            to_address: transfer.to_address,
            amount: transfer.value,
            currency: transfer.symbol,
            transaction_type: "credit".to_string(),
            status: "success".to_string(),
            metadata: TransactionMetadata::get_transaction_metadata(transfer.transaction_hash),
            exponent: 0,
        }
    }
}

impl From<Transactions> for UserTransaction {
    fn from(transfer: Transactions) -> UserTransaction {
        UserTransaction {
            user_address: transfer.to_address.clone(),
            transaction_id: Utils::generate_txn_id(),
            from_address: transfer.from_address,
            to_address: transfer.to_address,
            amount: transfer.value,
            currency: CONFIG.get_native_currency().to_string(),
            transaction_type: "credit".to_string(),
            status: "success".to_string(),
            metadata: TransactionMetadata::get_transaction_metadata(transfer.transaction_hash),
            exponent: 0,
        }
    }
}

impl UserTransaction {
    pub fn insert(_transactions: Vec<UserTransaction>) {
        unimplemented!();
    }

    pub fn from_token_transfers(transfers: Vec<TokenTransfers>) -> Vec<UserTransaction> {
        let mut user_transactions: Vec<UserTransaction> = vec![];
        for transfer in transfers {
            user_transactions.push(UserTransaction::from(transfer));
        }
        user_transactions
    }

    pub fn from_transactions(transfers: Vec<Transactions>) -> Vec<UserTransaction> {
        let mut user_transactions: Vec<UserTransaction> = vec![];
        for transfer in transfers {
            user_transactions.push(UserTransaction::from(transfer));
        }
        user_transactions
    }
}
