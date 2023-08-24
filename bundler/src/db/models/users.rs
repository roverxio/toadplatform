use sqlx::postgres::PgRow;
use sqlx::{FromRow, Row};

#[derive(FromRow, Debug)]
pub struct User {
    pub email: String,
    pub wallet_address: String,
    pub salt: String,
    pub deployed: bool,
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
