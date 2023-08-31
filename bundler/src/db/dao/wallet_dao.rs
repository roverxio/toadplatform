use log::{error, warn};
use sqlx::{query, query_as, Error, Pool, Postgres};

#[derive(Clone)]
pub struct WalletDao {
    pub pool: Pool<Postgres>,
}

impl WalletDao {
    pub async fn update_wallet_deployed(&self, user_id: String) {
        let query = query!(
            "UPDATE users SET deployed = $1 WHERE email = $2",
            true,
            user_id
        );
        let result = query.execute(&self.pool).await;
        if result.is_err() {
            warn!(
                "Failed to update deployed status for user: {}, err: {:?}",
                user_id,
                result.err()
            );
        }
    }

    pub async fn get_wallet_address(&self, user_id: String) -> String {
        let query = query_as!(User, "SELECT * from users where email = $1", user_id);
        let result: Result<User, Error> = query.fetch_one(&self.pool).await;
        return match result {
            Ok(user) => user.wallet_address,
            Err(err) => {
                error!("Failed to get wallet address {}, err: {:?}", user_id, err);
                "".to_string()
            }
        };
    }

    pub async fn get_wallet(&self, user_id: String) -> Option<User> {
        let query = query_as!(User, "SELECT * from users where email = $1", user_id);
        let result: Result<Option<User>, Error> = query.fetch_optional(&self.pool).await;
        return match result {
            Ok(user) => user,
            Err(err) => {
                error!("Failed to get wallet address {}, err: {:?}", user_id, err);
                None
            }
        };
    }

    pub async fn create_wallet(
        &self,
        user_id: String,
        wallet_address: String,
        salt: String,
        deployed: bool,
    ) {
        let query = query!(
            "INSERT INTO users (email, wallet_address, salt, deployed) VALUES ($1, $2, $3, $4)",
            user_id,
            wallet_address,
            salt,
            deployed
        );
        let result = query.execute(&self.pool).await;
        if result.is_err() {
            warn!(
                "Failed to create user: {}, err: {:?}",
                user_id,
                result.err()
            );
        }
    }
}

#[derive(Clone, Debug)]
pub struct User {
    pub email: String,
    pub wallet_address: String,
    pub salt: String,
    pub deployed: bool,
}
