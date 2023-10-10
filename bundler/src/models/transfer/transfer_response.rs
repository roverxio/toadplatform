use serde::Serialize;

use crate::models::transfer::TransactionResponse;

#[derive(Serialize)]
pub struct TransferResponse {
    pub transaction: TransactionResponse,
    pub transaction_id: String,
}
