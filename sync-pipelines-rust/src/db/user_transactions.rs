use crate::CONFIG;
use bigdecimal::BigDecimal;
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Pool, Postgres, QueryBuilder};
use std::process::exit;

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

pub struct UserTransactionWithJson {
    pub user_address: String,
    pub transaction_id: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: BigDecimal,
    pub currency: String,
    pub transaction_type: String,
    pub status: String,
    pub metadata: Value,
}

impl From<TokenTransfers> for UserTransaction {
    fn from(transfer: TokenTransfers) -> UserTransaction {
        UserTransaction {
            user_address: transfer.to_address.clone().unwrap_or("".to_string()),
            transaction_id: Utils::generate_txn_id(),
            from_address: transfer.from_address.unwrap_or("".to_string()),
            to_address: transfer.to_address.unwrap_or("".to_string()),
            amount: transfer.value.unwrap_or(BigDecimal::from(0)),
            currency: transfer.symbol.unwrap_or("".to_string()),
            transaction_type: "credit".to_string(),
            status: "success".to_string(),
            metadata: TransactionMetadata::get_transaction_metadata(
                transfer.transaction_hash.unwrap_or("".to_string()),
            ),
            exponent: 0,
        }
    }
}

impl From<Transactions> for UserTransaction {
    fn from(transfer: Transactions) -> UserTransaction {
        UserTransaction {
            user_address: transfer.to_address.clone().unwrap_or("".to_string()),
            transaction_id: Utils::generate_txn_id(),
            from_address: transfer.from_address.unwrap_or("".to_string()),
            to_address: transfer.to_address.unwrap_or("".to_string()),
            amount: transfer.value.unwrap_or(BigDecimal::from(0)),
            currency: CONFIG.get_native_currency().to_string(),
            transaction_type: "credit".to_string(),
            status: "success".to_string(),
            metadata: TransactionMetadata::get_transaction_metadata(
                transfer.transaction_hash.unwrap_or("".to_string()),
            ),
            exponent: 0,
        }
    }
}

impl From<UserTransaction> for UserTransactionWithJson {
    fn from(transaction: UserTransaction) -> UserTransactionWithJson {
        UserTransactionWithJson {
            user_address: transaction.user_address,
            transaction_id: transaction.transaction_id.clone(),
            from_address: transaction.from_address,
            to_address: transaction.to_address,
            amount: transaction.amount,
            currency: transaction.currency,
            transaction_type: transaction.transaction_type,
            status: transaction.status,
            metadata: match serde_json::to_value(&transaction.metadata) {
                Ok(data) => data,
                Err(err) => {
                    error!(
                        "Metadata conversion failed: {}, err: {:?}",
                        transaction.transaction_id, err
                    );
                    exit(1);
                }
            },
        }
    }
}

impl UserTransactionWithJson {
    pub fn get_user_transactions_with_json(
        transactions: Vec<UserTransaction>,
    ) -> Vec<UserTransactionWithJson> {
        let mut user_transactions_with_json: Vec<UserTransactionWithJson> = vec![];
        for transaction in transactions {
            user_transactions_with_json.push(UserTransactionWithJson::from(transaction));
        }
        user_transactions_with_json
    }
}

impl UserTransaction {
    pub async fn insert(pool: Pool<Postgres>, transactions: Vec<UserTransaction>) {
        let transactions = UserTransactionWithJson::get_user_transactions_with_json(transactions);
        let mut query_builder = QueryBuilder::new(
            "INSERT INTO user_transactions (user_address, transaction_id, from_address, \
            to_address, amount, currency, type, status, metadata) ",
        );
        query_builder.push_values(transactions, |mut b, txn| {
            b.push_bind(txn.user_address)
                .push_bind(txn.transaction_id)
                .push_bind(txn.from_address)
                .push_bind(txn.to_address)
                .push_bind(txn.amount)
                .push_bind(txn.currency)
                .push_bind(txn.transaction_type)
                .push_bind(txn.status)
                .push_bind(txn.metadata);
        });

        let query = query_builder.build();
        let res = query.execute(&pool).await;
        match res {
            Ok(_) => return,
            Err(e) => {
                error!("Unable to insert into user_transactions: {}", e);
                exit(1);
            }
        }
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
