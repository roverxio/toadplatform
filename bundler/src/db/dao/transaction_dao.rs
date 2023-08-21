use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde_json::json;

use crate::db::dao::connect::connect;
use crate::models::transfer::transfer_request::TransferRequest;

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

    pub async fn create_user_transaction(
        &self,
        user_address: String,
        transaction_id: String,
        from_address: String,
        request: TransferRequest,
        txn_type: String,
        status: String,
    ) {
        unimplemented!();
    }
}
