use serde::Deserialize;

#[derive(Deserialize)]
pub struct PollTransactionStatusParams {
    pub transaction_id: String,
}
