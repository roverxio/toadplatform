use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct UserDao {
    pub pool: Pool<Postgres>,
}

impl UserDao {
    pub async fn get(&self) {
        let query = sqlx::query!("SELECT * FROM users");
        let result = query.fetch_all(&self.pool).await?;
        for row in result {
            println!("{:?}", row)
        }
    }
}
