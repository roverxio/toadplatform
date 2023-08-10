use r2d2::Pool;
use r2d2_sqlite::rusqlite::Statement;
use r2d2_sqlite::SqliteConnectionManager;

use crate::db::dao::connect::connect;

#[derive(Clone)]
pub struct WalletDao {
    pub pool: Pool<SqliteConnectionManager>,
}

impl WalletDao {
    pub async fn update_wallet_deployed(&self, user_id: String) {
        let conn = connect(self.pool.clone()).await;

        let mut stmt = conn
            .prepare("UPDATE users SET deployed = ? WHERE email = ?")
            .unwrap();
        stmt.execute([true.to_string(), user_id]).unwrap();
    }

    pub async fn get_wallet_address(&self, user_id: String) -> String {
        let conn = connect(self.pool.clone()).await;

        let mut stmt = conn
            .prepare("SELECT * from users where email = ? limit 1")
            .unwrap();
        let rows = Self::get_user(user_id, &mut stmt);

        if !rows.is_empty() {
            return rows[0].wallet_address.to_string();
        }
        return "".to_string();
    }

    pub async fn get_wallet(&self, user_id: String) -> Option<User> {
        let conn = connect(self.pool.clone()).await;
        let mut stmt = conn
            .prepare("SELECT * from users where email = ? limit 1")
            .unwrap();
        let rows = Self::get_user(user_id, &mut stmt);

        if !rows.is_empty() {
            return Some(User {
                email: rows[0].email.to_string(),
                wallet_address: rows[0].wallet_address.to_string(),
                salt: rows[0].salt.clone(),
                deployed: rows[0].deployed,
            });
        }
        return None;
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

#[derive(Debug)]
pub struct User {
    pub email: String,
    pub wallet_address: String,
    pub salt: String,
    pub deployed: bool,
}
