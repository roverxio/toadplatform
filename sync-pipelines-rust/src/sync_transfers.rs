use crate::utils::{generate_txn_id, get_last_synced_time};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use std::process::exit;

pub enum Table {
    TokenTransfers,
    Transactions,
}

impl Table {
    pub fn to_string(&self) -> String {
        match self {
            Table::TokenTransfers => "token_transfers".to_string(),
            Table::Transactions => "transactions".to_string(),
        }
    }

    pub fn from(s: &str) -> Self {
        match s {
            "token_transfers" => Table::TokenTransfers,
            "transactions" => Table::Transactions,
            _ => {
                // raise error
                exit(1);
            }
        }
    }
}

pub fn get_transfers(from_table: Table) -> Vec<Transfer> {
    // get last_synced_time
    let _last_sync_time = get_last_synced_time(from_table.to_string());
    // sqlx query to get relevant transfers - using users(wallet_addresses) x token_transfers(from last synced time)
    unimplemented!();
}

pub fn get_transactions(from_table: Table) -> Vec<Transfer> {
    // get last_synced_time
    let _last_sync_time = get_last_synced_time(from_table.to_string());
    // sqlx query to get relevant transactions - using users(wallet_addresses) x transactions(from last synced time)
    unimplemented!();
}

pub fn insert_user_transactions(_user_transactions: Vec<UserTransaction>) {
    // sqlx query to insert into user_transactions table
    unimplemented!();
}

pub fn sync_user_transactions(from_table: Table) {
    // get last_synced_time
    let _last_sync_time = get_last_synced_time(from_table.to_string());
    // sqlx query to get relevant transfers - using users(wallet_addresses) x token_transfers(from last synced time)
    let transfers = match from_table {
        Table::TokenTransfers => get_transfers(from_table),
        Table::Transactions => get_transactions(from_table),
    };

    let mut user_transactions: Vec<UserTransaction> = Vec::new();
    for transfer in transfers {
        user_transactions.push(UserTransaction::from(transfer))
    }

    insert_user_transactions(user_transactions);
}

pub struct Transfer {
    pub user_address: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: BigDecimal,
    pub currency: String,         // config("native_currency")
    pub transaction_type: String, // = "credit"
    pub status: String,           // = "success"
    pub metadata: TransactionMetadata,
    pub exponent: i32,
}

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

impl From<Transfer> for UserTransaction {
    fn from(transfer: Transfer) -> Self {
        Self {
            user_address: transfer.user_address,
            transaction_id: generate_txn_id(),
            from_address: transfer.from_address,
            to_address: transfer.to_address,
            amount: transfer.amount,
            currency: transfer.currency, // config("native_currency")
            transaction_type: transfer.transaction_type, // = "credit"
            status: transfer.status,     // = "success"
            metadata: transfer.metadata,
            exponent: transfer.exponent,
        }
    }
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
