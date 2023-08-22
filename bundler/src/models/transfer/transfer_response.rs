use crate::models::transfer::transaction_response::TransactionResponse;
use serde::Serialize;

#[derive(Serialize)]
pub struct TransferResponse {
    pub transaction: TransactionResponse,
    // pub transaction_id: String,
}
