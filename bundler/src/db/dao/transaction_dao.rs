use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::db::dao::connect::connect;

#[derive(Clone)]
pub struct TransactionDao {
    pub pool: Pool<SqliteConnectionManager>,
}

impl TransactionDao {
    pub async fn create_transaction(&self, wallet_address: String, transaction_hash: String) {
        let conn = connect(self.pool.clone()).await;

        let mut stmt = conn
            .prepare("INSERT INTO transactions (wallet_address, transaction_hash) VALUES (?, ?)")
            .unwrap();
        stmt.execute([wallet_address, transaction_hash]).unwrap();
    }

    pub async fn update_user_transactions(
        &self,
        _txn_id: String,
        _txn_hash: String,
        _status: String,
    ) {
        // function to update user_transactions table
    }
}
