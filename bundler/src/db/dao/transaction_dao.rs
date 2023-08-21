use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

#[derive(Clone)]
pub struct TransactionDao {
    pub pool: Pool<SqliteConnectionManager>,
}

impl TransactionDao {
    pub async fn create_user_transaction(
        &self,
        _user_address: String,
        _transaction_id: String,
        _from_address: String,
        _to_address: String,
        _amount: f64,
        _currency: String,
        _txn_type: String,
        _status: String,
    ) {
        unimplemented!();
    }
}
