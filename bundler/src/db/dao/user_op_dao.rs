use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

#[derive(Clone)]
pub struct UserOpDao {
    pub pool: Pool<SqliteConnectionManager>,
}

impl UserOpDao {
    pub async fn create_user_op(&self, txn_id: String, user_op_hash: String) {
        unimplemented!();
    }
}
