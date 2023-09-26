use crate::CONFIG;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, QueryBuilder};

use crate::db::token_transfers::TokenTransfers;
use crate::db::transactions::Transactions;
use crate::utils::status::Status::SUCCESS;
use crate::utils::transaction_type::TransactionType::CREDIT;
use crate::utils::utils::Utils;

#[derive(Clone, Default, PartialEq)]
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
}

#[derive(Default, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct Gas {
    pub currency: String,
    pub value: u64,
}

impl From<TokenTransfers> for UserTransaction {
    fn from(transfer: TokenTransfers) -> UserTransaction {
        UserTransaction {
            user_address: transfer.to_address.clone().unwrap(),
            transaction_id: Utils::generate_txn_id(),
            from_address: transfer.from_address.unwrap(),
            to_address: transfer.to_address.unwrap(),
            amount: transfer.value.unwrap(),
            currency: transfer.symbol.unwrap(),
            transaction_type: CREDIT.to_string(),
            status: SUCCESS.to_string(),
            metadata: TransactionMetadata::get_transaction_metadata(transfer.transaction_hash),
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
            transaction_type: CREDIT.to_string(),
            status: SUCCESS.to_string(),
            metadata: TransactionMetadata::get_transaction_metadata(
                transfer.transaction_hash.unwrap_or("".to_string()),
            ),
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
            to_address, amount, currency, type, status, metadata) VALUES",
        );
        for txn in transactions.iter() {
            let metadata_value = match serde_json::to_value(&txn.metadata) {
                Ok(data) => data,
                Err(err) => {
                    return Err(format!(
                        "Metadata conversion failed: {}, err: {:?}",
                        txn.transaction_id, err
                    ));
                }
            };

            query_builder.push(format!(
                "('{}','{}','{}','{}',{},'{}','{}','{}','{}')",
                txn.user_address,
                txn.transaction_id,
                txn.from_address,
                txn.to_address,
                txn.amount,
                txn.currency,
                txn.transaction_type,
                txn.status,
                metadata_value,
            ));

            if Some(txn) == transactions.last() {
                query_builder.push(";");
            } else {
                query_builder.push(",");
            }
        }

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
