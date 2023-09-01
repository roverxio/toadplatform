use log::error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{query, Pool, Postgres};

#[derive(Clone)]
pub struct TransactionDao {
    pub pool: Pool<Postgres>,
}

impl TransactionDao {
    pub async fn list_transactions(
        &self,
        page_size: i64,
        id: i32,
        user_wallet: String,
    ) -> Vec<UserTransactionWithExponent> {
        let query = query!(
            "SELECT t1.*, t1.type as transaction_type, t2.exponent from user_transactions t1 left \
            join token_metadata t2 on t1.currency = t2.symbol \
            where user_address = $1 and id < $2 order by id desc limit $3",
            user_wallet,
            id,
            page_size
        );
        let result = query.fetch_all(&self.pool).await;
        return match result {
            Ok(rows) => {
                let mut transactions = Vec::new();
                for row in rows {
                    let metadata: TransactionMetadata;
                    match serde_json::from_value(row.metadata) {
                        Ok(data) => metadata = data,
                        Err(err) => {
                            error!(
                                "Metadata deserialization failed: {}, err: {:?}",
                                row.transaction_id, err
                            );
                            continue;
                        }
                    }
                    transactions.push(UserTransactionWithExponent {
                        user_transaction: UserTransaction {
                            id: row.id,
                            user_address: row.user_address,
                            transaction_id: row.transaction_id,
                            from_address: row.from_address,
                            to_address: row.to_address,
                            amount: row.amount,
                            currency: row.currency,
                            transaction_type: row.transaction_type,
                            status: row.status,
                            metadata,
                            created_at: row.created_at.to_string(),
                            updated_at: row.updated_at.to_string(),
                        },
                        exponent: row.exponent,
                    })
                }
                transactions
            }
            Err(error) => {
                error!("Failed to fetch transactions: {:?}", error);
                vec![]
            }
        };
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
}

#[derive(Clone, Default)]
pub struct UserTransaction {
    pub id: i32,
    pub user_address: String,
    pub transaction_id: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: String,
    pub currency: String,
    pub transaction_type: String,
    pub status: String,
    pub metadata: TransactionMetadata,
    pub created_at: String,
    pub updated_at: String,
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

    pub fn transaction_hash(&mut self, transaction_hash: String) -> &mut TransactionMetadata {
        self.transaction_hash = transaction_hash;
        self
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

    pub fn amount(&mut self, amount: String) -> &mut UserTransaction {
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

pub struct UserTransactionWithExponent {
    pub user_transaction: UserTransaction,
    pub exponent: i32,
}
