use serde::Deserialize;

#[derive(Deserialize)]
pub struct PollTransactionParams {
    pub transaction_id: String,
}
