use crate::db::models::users::User;
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct UserDao {
    pub pool: Pool<Postgres>,
}

impl UserDao {
    pub async fn get(&self) {
        let query = sqlx::query(r#"SELECT * FROM users"#);
        let result: Vec<User> = query.map(User::new).fetch_all(&self.pool).await.unwrap();
        for row in result {
            println!("{:?}", row)
        }
    }
}
