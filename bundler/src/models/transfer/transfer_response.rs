use serde::Serialize;

#[derive(Serialize)]
pub struct TransactionResponse {
    pub transaction_hash: String,
    pub status: String,
    pub explorer: String,
}
