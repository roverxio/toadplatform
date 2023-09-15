use crate::CONFIG;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
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
            exponent: transfer.exponent.unwrap_or(0),
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
            exponent: transfer.exponent.unwrap_or(0),
        }
    }
}

impl UserTransaction {
    pub async fn insert(
        pool: Pool<Postgres>,
        transactions: Vec<UserTransaction>,
    ) -> Result<(), String> {
        let mut query_builder = QueryBuilder::new(
            "INSERT INTO user_transactions (user_address, transaction_id, from_address, \
            to_address, amount, currency, type, status, metadata, exponent) ",
        );
        query_builder.push_values(transactions, |mut b, txn| {
            b.push_bind(txn.user_address)
                .push_bind(txn.transaction_id.clone())
                .push_bind(txn.from_address)
                .push_bind(txn.to_address)
                .push_bind(txn.amount)
                .push_bind(txn.currency)
                .push_bind(txn.transaction_type)
                .push_bind(txn.status)
                .push_bind(match serde_json::to_value(&txn.metadata) {
                    Ok(data) => data,
                    Err(err) => {
                        format!(
                            "Metadata conversion failed: {}, err: {:?}",
                            txn.transaction_id, err
                        );
                        exit(1);
                    }
                })
                .push_bind(txn.exponent);
        });

        let query = query_builder.build();
        let res = query.execute(&pool).await;
        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to insert into user_transactions: {}", e)),
        }
    }

    pub fn from_token_transfers(token_transfers: Vec<TokenTransfers>) -> Vec<UserTransaction> {
        token_transfers
            .into_iter()
            .map(UserTransaction::from)
            .collect()
    }

    pub fn from_transactions(transfers: Vec<Transactions>) -> Vec<UserTransaction> {
        transfers.into_iter().map(UserTransaction::from).collect()
    }
}
