use chrono::{DateTime, Utc};
use log::error;
use serde_json::Value;
use sqlx::types::JsonValue;
use sqlx::{query, query_as, Pool, Postgres};
use std::default::Default;

use crate::errors::DatabaseError;
use crate::models::contract_interaction::user_operation::UserOperation;

#[derive(Clone)]
pub struct UserOperationDao {
    pub pool: Pool<Postgres>,
}

impl UserOperationDao {
    pub async fn create_user_operation(
        pool: &Pool<Postgres>,
        transaction_id: String,
        user_operation: UserOperation,
        status: String,
    ) -> Result<(), DatabaseError> {
        let metadata: Value;
        match serde_json::to_value(user_operation) {
            Ok(data) => metadata = data,
            Err(err) => {
                return Err(DatabaseError::ServerError(format!(
                    "UserOperation conversion failed: {}, err: {:?}",
                    transaction_id, err
                )));
            }
        }
        let query = query!(
            "INSERT INTO user_operations (transaction_id, user_operation, status) VALUES \
                ($1, $2, $3)",
            transaction_id,
            metadata,
            status,
        );
        let result = query.execute(pool).await;
        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(DatabaseError::ServerError(format!(
                "Failed to create user operation: {}, err: {:?}",
                transaction_id, err
            ))),
        }
    }

    pub async fn get_user_operation(&self, transaction_id: String) -> UserOperationRecord {
        let query = query_as!(
            UserOperationRecord,
            "SELECT * from user_operations where transaction_id = $1",
            transaction_id
        );
        let result = query.fetch_one(&self.pool).await;
        match result {
            Ok(row) => row,
            Err(error) => {
                error!("Failed to fetch user operation: {:?}", error);
                Default::default()
            }
        }
    }

    pub async fn update_user_operation_status(
        &self,
        transaction_id: String,
        status: String,
    ) -> bool {
        let query = query!(
            "UPDATE user_operations SET status = $1 where transaction_id = $2",
            status,
            transaction_id
        );
        let result = query.execute(&self.pool).await;
        if result.is_err() {
            error!(
                "Failed to update user operation status: {}, err: {:?}",
                transaction_id,
                result.err()
            );
            return false;
        }
        true
    }
}

#[derive(Clone, Default)]
pub struct UserOperationRecord {
    pub transaction_id: String,
    pub user_operation: UserOperation,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<JsonValue> for UserOperation {
    fn from(json: JsonValue) -> Self {
        serde_json::from_value(json).unwrap()
    }
}
