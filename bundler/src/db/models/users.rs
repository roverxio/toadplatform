use sqlx::postgres::PgRow;
use sqlx::{FromRow, Row};

#[derive(FromRow, Debug)]
pub struct User {
    pub email: Option<String>,
    pub wallet_address: Option<String>,
    pub salt: Option<String>,
    pub deployed: Option<bool>,
}

impl User {
    pub fn new(row: PgRow) -> User {
        User {
            email: row.get("email"),
            wallet_address: row.get("wallet_address"),
            salt: row.get("salt"),
            deployed: row.get("deployed"),
        }
    }
}
