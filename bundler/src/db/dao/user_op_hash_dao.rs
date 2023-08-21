use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

#[derive(Clone)]
pub struct UserOpHashDao {
    pub pool: Pool<SqliteConnectionManager>,
}

impl UserOpHashDao {
    pub async fn create_user_op(&self, _txn_id: String, _user_op_hash: String) {
        unimplemented!();
    }
}
