use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde::{Deserialize, Serialize};

use crate::db::dao::connect::connect;

#[derive(Clone)]
pub struct TransactionDao {
    pub pool: Pool<SqliteConnectionManager>,
}

impl TransactionDao {
    pub async fn create_user_transaction(&self, txn: UserTransaction) {
        let conn = connect(self.pool.clone()).await;
        let mut stmt = conn
            .prepare(
                "INSERT INTO user_transactions (user_address, transaction_id, from_address, to_address, amount, currency, type, status, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .unwrap();
        stmt.execute([
            txn.user_address.clone(),
            txn.transaction_id.clone(),
            txn.from_address.clone(),
            txn.to_address.clone(),
            txn.amount.clone(),
            txn.currency.clone(),
            txn.transaction_type.clone(),
            txn.status.clone(),
            serde_json::to_string(&txn.metadata).unwrap(),
        ])
        .unwrap();
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
