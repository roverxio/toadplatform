use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::JsonValue;
use sqlx::{query, query_as, Pool, Postgres};
use std::default::Default;

use crate::errors::base::DatabaseError;

#[derive(Clone)]
pub struct TransactionDao {
    pub pool: Pool<Postgres>,
}

impl TransactionDao {
    pub async fn list_transactions(
        pool: &Pool<Postgres>,
        page_size: i64,
        id: i32,
        user_wallet: String,
    ) -> Result<Vec<UserTransaction>, DatabaseError> {
        let query = query_as!(
            UserTransaction,
            "SELECT t1.id, t1.user_address, t1.transaction_id, t1.from_address, t1.to_address, \
            t1.amount, t1.currency, t1.type as transaction_type, t1.status, t1.metadata, \
            t1.created_at, t1.updated_at, t2.exponent from user_transactions t1 left join \
            token_metadata t2 on lower(t1.currency) = lower(t2.symbol) \
            and lower(t1.metadata ->> 'chain') = lower(t2.chain) \
            where user_address = $1 and id < $2 order by id desc limit $3",
            user_wallet,
            id,
            page_size
        );
        let result = query.fetch_all(pool).await;
        match result {
            Ok(rows) => Ok(rows),
            Err(error) => Err(DatabaseError(format!(
                "Failed to fetch transactions: {:?}",
                error
            ))),
        }
    }

    pub async fn create_user_transaction(&self, txn: UserTransaction) {
        let metadata: Value;
        match serde_json::to_value(&txn.metadata) {
            Ok(data) => metadata = data,
            Err(err) => {
                error!(
                    "Metadata conversion failed: {}, err: {:?}",
                    txn.transaction_id, err
                );
                return;
            }
        }
        let query = query!(
            "INSERT INTO user_transactions (user_address, transaction_id, from_address,\
                to_address, amount, currency, type, status, metadata) VALUES \
                ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            txn.user_address.clone(),
            txn.transaction_id.clone(),
            txn.from_address.clone(),
            txn.to_address.clone(),
            txn.amount.clone(),
            txn.currency.clone(),
            txn.transaction_type.clone(),
            txn.status.clone(),
            metadata
        );
        let result = query.execute(&self.pool).await;
        if result.is_err() {
            error!(
                "Failed to create user transaction: {}, err: {:?}",
                txn.transaction_id,
                result.err()
            );
        }
    }

    pub async fn get_transaction_by_id(
        pool: &Pool<Postgres>,
        txn_id: String,
        user_wallet_address: String,
    ) -> Result<UserTransaction, String> {
        let query = query_as!(
            UserTransaction,
            "SELECT t1.id, t1.user_address, t1.transaction_id, t1.from_address, \
            t1.to_address, t1.amount, t1.currency, t1.type as transaction_type, \
            t1.status, t1.metadata, t1.created_at, t1.updated_at, t2.exponent \
            from user_transactions t1 left join token_metadata t2 \
            on lower(t1.currency) = lower(t2.symbol) and \
            lower(t1.metadata ->> 'chain') = lower(t2.chain) \
            where transaction_id = $1 and user_address = $2",
            txn_id,
            user_wallet_address,
        );
        let result = query.fetch_one(pool).await;
        match result {
            Ok(row) => Ok(row),
            Err(error) => Err(format!("Failed to fetch transactions: {:?}", error)),
        }
    }

    pub async fn update_user_transaction(
        &self,
        txn_id: String,
        txn_hash: Option<String>,
        status: String,
    ) {
        let query;
        match txn_hash {
            None => {
                query = query!(
                    "UPDATE user_transactions set status = $1 where transaction_id = $2",
                    status,
                    txn_id,
                );
            }
            Some(value) => {
                query = query!(
                    "UPDATE user_transactions \
                    set status = $1, metadata = jsonb_set(metadata, '{transaction_hash}', $2) \
                    where transaction_id = $3",
                    status,
                    Value::String(value),
                    txn_id,
                );
            }
        }
        let result = query.execute(&self.pool).await;
        if result.is_err() {
            error!(
                "Failed to update user transaction: {}, err: {:?}",
                txn_id,
                result.err()
            );
        }
    }
}

#[derive(Clone, Default)]
pub struct UserTransaction {
    pub id: i32,
    pub user_address: String,
    pub transaction_id: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: BigDecimal,
    pub currency: String,
    pub transaction_type: String,
    pub status: String,
    pub metadata: TransactionMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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

impl TransactionMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn chain(&mut self, chain: String) -> &mut TransactionMetadata {
        self.chain = chain;
        self
    }
}

impl From<JsonValue> for TransactionMetadata {
    fn from(json: JsonValue) -> Self {
        serde_json::from_value(json).unwrap()
    }
}

impl UserTransaction {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn user_address(&mut self, user_address: String) -> &mut UserTransaction {
        self.user_address = user_address;
        self
    }

    pub fn transaction_id(&mut self, transaction_id: String) -> &mut UserTransaction {
        self.transaction_id = transaction_id;
        self
    }

    pub fn sender_address(&mut self, from_address: String) -> &mut UserTransaction {
        self.from_address = from_address;
        self
    }

    pub fn receiver_address(&mut self, to_address: String) -> &mut UserTransaction {
        self.to_address = to_address;
        self
    }

    pub fn amount(&mut self, amount: BigDecimal) -> &mut UserTransaction {
        self.amount = amount;
        self
    }

    pub fn currency(&mut self, currency: String) -> &mut UserTransaction {
        self.currency = currency;
        self
    }

    pub fn transaction_type(&mut self, transaction_type: String) -> &mut UserTransaction {
        self.transaction_type = transaction_type;
        self
    }

    pub fn status(&mut self, status: String) -> &mut UserTransaction {
        self.status = status;
        self
    }

    pub fn metadata(&mut self, metadata: TransactionMetadata) -> &mut UserTransaction {
        self.metadata = metadata;
        self
    }
}
