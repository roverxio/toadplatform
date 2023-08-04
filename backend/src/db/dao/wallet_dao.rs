use actix_web::{error, web};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;

#[derive(Clone)]
pub struct WalletDao {
    pub pool: Pool<SqliteConnectionManager>,
}

impl WalletDao {
    pub async fn connect(&self) -> PooledConnection<SqliteConnectionManager> {
        let pool1 = self.pool.clone();
        let conn = web::block(move || pool1.get()).await.unwrap().map_err(error::ErrorInternalServerError).unwrap(); // <- create async connection (non-blocking
        return conn;
    }

    pub async fn get_wallet(&self, user_id: String) -> String {
        let conn  = self.connect().await;

        let mut stmt = conn.prepare("SELECT * from users where email = ? limit 1").unwrap();
        let rows: Vec<User> = stmt.query_map([user_id], |row| {
            Ok(
                User {
                    email: row.get(0)?,
                    wallet_address: row.get(1)?,
                }
            )
        }).and_then(Iterator::collect).unwrap();

        if !rows.is_empty() {
            return rows[0].wallet_address.to_string();
        }
        return "".to_string();
    }

    pub async fn create_wallet(&self, user_id: String, wallet_address: String) {
        let conn  = self.connect().await;

        let mut stmt = conn.prepare("INSERT INTO users (email, wallet_address) VALUES (?, ?)").unwrap();
        stmt.execute([user_id, wallet_address]).unwrap();
    }
}

#[derive(Debug)]
pub struct User {
    pub email: String,
    pub wallet_address: String,
}
