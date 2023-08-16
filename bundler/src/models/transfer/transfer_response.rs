use crate::models::transfer::status::Status;
use crate::models::transfer::transaction_response::TransactionResponse;
use serde::Serialize;

#[derive(Serialize)]
pub struct TransferResponse {
    pub transaction: TransactionResponse,
}

impl TransactionResponse {
    pub fn new(transaction_hash: String, status: Status, explorer: String) -> TransactionResponse {
        TransactionResponse {
            transaction_hash,
            status: status.to_string(),
            explorer,
        }
    }
}
