use crate::models::contract_interaction::user_operation::UserOperation;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct SigningPayload {
    pub user_operation: UserOperation,
    pub entrypoint_address: String,
    pub chain_id: u64,
}
