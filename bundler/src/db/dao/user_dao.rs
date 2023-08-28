use crate::db::models::users::User;
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct UserDao {
    pub pool: Pool<Postgres>,
}

impl UserDao {
    pub async fn init(&self) {
        println!("------------ Using query function ------------");
        self.get_function().await;
        println!("------------ Using query as macro ------------");
        self.get_macro_as().await;
        println!("------------ Using query macro ------------");
        self.get_macro().await;
    }

    pub async fn get_function(&self) {
        let query = sqlx::query(r#"SELECT * FROM users"#);
        let result: Vec<User> = query.map(User::new).fetch_all(&self.pool).await.unwrap();
        for row in result {
            println!("{:?}", row)
        }
    }

    pub async fn get_macro_as(&self) {
        let query = sqlx::query_as!(User, r#"SELECT * FROM users"#);
        let result: Vec<User> = query.fetch_all(&self.pool).await.unwrap();
        for row in result {
            println!("{:?}", row)
        }
    }

    pub async fn get_macro(&self) {
        let query = sqlx::query!(r#"SELECT * FROM users"#);
        let result = query.fetch_all(&self.pool).await.unwrap();
        for row in result {
            println!(
                "{:?}",
                User {
                    email: row.email,
                    wallet_address: row.wallet_address,
                    salt: row.salt,
                    deployed: row.deployed,
                }
            )
        }
    }
}
