use r2d2::Pool;
use r2d2_sqlite::rusqlite::{params, Result, Row, ToSql};
use r2d2_sqlite::SqliteConnectionManager;
use serde::{Deserialize, Serialize};

use crate::db::dao::connect::connect;

#[derive(Clone)]
pub struct TransactionDao {
    pub pool: Pool<SqliteConnectionManager>,
}

impl TransactionDao {
    pub async fn list_transactions(
        &self,
        page_size: i32,
        id: i32,
        user_wallet: String,
    ) -> Vec<UserTransactionWithExponent> {
        let query = "SELECT t1.id, t1.user_address, t1.transaction_id, t1.from_address, \
            t1.to_address, t1.amount, t1.currency, t1.type, t1.status, t1.metadata, t1.created_at, \
            t1.updated_at, t2.exponent from user_transactions t1 left join supported_currencies t2 on \
            t1.currency = t2.currency where user_address = ? and id < ? order by id desc limit ?".to_string();

        let params = params![user_wallet, id.to_string(), page_size.to_string()];
        get_user_transactions(&self.pool.clone(), query, params).await
    }

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

pub async fn get_transaction_by_id(
    pool: &Pool<SqliteConnectionManager>,
    txn_id: String,
) -> UserTransactionWithExponent {
    let query = "SELECT id, user_address, transaction_id, from_address, to_address, amount, currency, type, status, metadata, created_at, updated_at FROM user_transactions WHERE transaction_id = ?".to_string();
    let params = params![txn_id];
    let user_transaction_data = get_user_transactions(pool, query, params).await;
    if user_transaction_data.is_empty() {
        Default::default()
    } else {
        let transaction = user_transaction_data.get(0).unwrap();
        (*transaction).clone()
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

#[derive(Default, Clone)]
pub struct UserTransactionWithExponent {
    pub user_transaction: UserTransaction,
    pub exponent: i32,
}

fn get_user_transaction_with_exponent(row: &Row) -> Result<UserTransactionWithExponent> {
    let metadata: TransactionMetadata =
        serde_json::from_str(&row.get::<_, String>(9).unwrap()).unwrap();
    let amount: String = row.get(5).unwrap();
    let mut exponent = row.get(12);
    if exponent.is_err() {
        exponent = Ok(0);
    }

    Ok(UserTransactionWithExponent {
        user_transaction: UserTransaction {
            id: row.get(0)?,
            user_address: row.get(1)?,
            transaction_id: row.get(2)?,
            from_address: row.get(3)?,
            to_address: row.get(4)?,
            amount,
            currency: row.get(6)?,
            transaction_type: row.get(7)?,
            status: row.get(8)?,
            metadata,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        },
        exponent: exponent?,
    })
}

async fn get_user_transactions(
    pool: &Pool<SqliteConnectionManager>,
    query: String,
    params: &[&dyn ToSql],
) -> Vec<UserTransactionWithExponent> {
    let conn = connect(pool.clone()).await;

    let mut stmt = conn.prepare(&query).unwrap();
    let rows: Vec<UserTransactionWithExponent> = stmt
        .query_map(params, |row| get_user_transaction_with_exponent(row))
        .and_then(Iterator::collect)
        .unwrap();
    rows
}
