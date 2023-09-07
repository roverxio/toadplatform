use crate::models::contract_interaction::user_operation::UserOperation;
use chrono::{DateTime, Utc};
use log::error;
use serde_json::Value;
use sqlx::types::JsonValue;
use sqlx::{query, Pool, Postgres};
use std::default::Default;

#[derive(Clone)]
pub struct UserOperationDao {
    pub pool: Pool<Postgres>,
}

impl UserOperationDao {
    pub async fn create_user_operation(
        &self,
        transaction_id: String,
        user_operation: UserOperation,
        status: String,
    ) {
        let metadata: Value;
        match serde_json::to_value(user_operation) {
            Ok(data) => metadata = data,
            Err(err) => {
                error!(
                    "Metadata conversion failed: {}, err: {:?}",
                    transaction_id, err
                );
                return;
            }
        }
        let query = query!(
            "INSERT INTO user_operations (transaction_id, user_operation, status) VALUES \
                ($1, $2, $3)",
            transaction_id,
            metadata,
            status,
        );
        let result = query.execute(&self.pool).await;
        if result.is_err() {
            error!(
                "Failed to create user transaction: {}, err: {:?}",
                transaction_id,
                result.err()
            );
        }
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
