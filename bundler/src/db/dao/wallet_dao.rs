use log::warn;
use r2d2::Pool;
use r2d2_sqlite::rusqlite::Statement;
use r2d2_sqlite::SqliteConnectionManager;
use sqlx::{query, query_as, Error, Postgres};

use crate::db::dao::connect::connect;

#[derive(Clone)]
pub struct WalletDao {
    pub pool: Pool<SqliteConnectionManager>,
    pub db_pool: sqlx::Pool<Postgres>,
}

impl WalletDao {
    pub async fn update_wallet_deployed(&self, user_id: String) {
        let query = query!(
            "UPDATE users SET deployed = $1 WHERE email = $2",
            true,
            user_id
        );
        let result = query.execute(&self.db_pool).await;
        if result.is_err() {
            warn!("Failed to update deployed status for user: {}", user_id);
        }
    }

    pub async fn get_wallet_address(&self, user_id: String) -> String {
        let query = query_as!(User, "SELECT * from users where email = $1", user_id);
        let result: Result<User, Error> = query.fetch_one(&self.db_pool).await;
        return match result {
            Ok(user) => user.wallet_address,
            Err(_) => "".to_string(),
        };
    }

    pub async fn get_wallet(&self, user_id: String) -> Option<User> {
        let query = query_as!(User, "SELECT * from users where email = $1", user_id);
        let result: Result<Option<User>, Error> = query.fetch_optional(&self.db_pool).await;
        return match result {
            Ok(user) => user,
            Err(_) => None,
        };
    }

    pub async fn create_wallet(
        &self,
        user_id: String,
        wallet_address: String,
        salt: String,
        deployed: bool,
    ) {
        let conn = connect(self.pool.clone()).await;

        let mut stmt = conn
            .prepare(
                "INSERT INTO users (email, wallet_address, salt, deployed) VALUES (?, ?, ?, ?)",
            )
            .unwrap();
        stmt.execute([user_id, wallet_address, salt, deployed.to_string()])
            .unwrap();
    }

    fn get_user(user_id: String, stmt: &mut Statement) -> Vec<User> {
        let rows: Vec<User> = stmt
            .query_map([user_id], |row| {
                let deployed_str: String = row.get(3)?;
                let deployed = match deployed_str.as_str() {
                    "true" => true,
                    "false" => false,
                    _ => false,
                };
                Ok(User {
                    email: row.get(0)?,
                    wallet_address: row.get(1)?,
                    salt: row.get(2)?,
                    deployed,
                })
            })
            .and_then(Iterator::collect)
            .unwrap();
        rows
    }
}

#[derive(Clone, Debug)]
pub struct User {
    pub email: String,
    pub wallet_address: String,
    pub salt: String,
    pub deployed: bool,
}
