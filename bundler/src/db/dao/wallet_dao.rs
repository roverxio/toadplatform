use bigdecimal::BigDecimal;
use log::error;
use sqlx::{query, query_as, Error, Pool, Postgres};

use crate::errors::base::DatabaseError;

#[derive(Clone)]
pub struct WalletDao {
    pub pool: Pool<Postgres>,
}

impl WalletDao {
    pub async fn update_wallet_deployed(&self, user_id: String) {
        let query = query!(
            "UPDATE users SET deployed = $1 WHERE external_user_id = $2",
            true,
            user_id
        );
        let result = query.execute(&self.pool).await;
        if result.is_err() {
            error!(
                "Failed to update deployed status for user: {}, err: {:?}",
                user_id,
                result.err()
            );
        }
    }

    pub async fn get_wallet_by_external_user_id(
        pool: &Pool<Postgres>,
        external_user_id: String,
    ) -> Option<User> {
        let query = query_as!(
            User,
            "SELECT * from users where external_user_id = $1",
            external_user_id
        );
        let result: Result<Option<User>, Error> = query.fetch_optional(pool).await;
        return match result {
            Ok(user) => user,
            Err(err) => {
                error!(
                    "Failed to get wallet address {}, err: {:?}",
                    external_user_id, err
                );
                None
            }
        };
    }

    pub async fn create_wallet(
        pool: &Pool<Postgres>,
        user_id: String,
        name: String,
        wallet_address: String,
        owner_address: String,
        external_user_id: String,
        salt: BigDecimal,
        deployed: bool,
    ) -> Result<(), DatabaseError> {
        let query = query!(
            "INSERT INTO users (email, name, wallet_address, owner_address, salt, external_user_id, \
            deployed) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            user_id,
            name,
            wallet_address.to_lowercase(),
            owner_address.to_lowercase(),
            salt,
            external_user_id,
            deployed
        );
        let result = query.execute(pool).await;
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(DatabaseError(format!(
                "Failed to create user: {}, err: {:?}",
                user_id,
                result.err()
            ))),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct User {
    pub email: String,
    pub wallet_address: String,
    pub salt: BigDecimal,
    pub deployed: bool,
    pub owner_address: String,
    pub name: String,
    pub external_user_id: String,
}

// mapper to convert from firebase user to db user
impl From<rs_firebase_admin_sdk::auth::User> for User {
    fn from(user: rs_firebase_admin_sdk::auth::User) -> Self {
        User {
            email: user.email.unwrap(),
            wallet_address: Default::default(),
            salt: Default::default(),
            deployed: false,
            owner_address: Default::default(),
            name: user.display_name.unwrap(),
            external_user_id: user.uid,
        }
    }
}
