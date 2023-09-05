use serde::Deserialize;

#[derive(Deserialize)]
pub struct PollTransactionParams {
    pub transaction_id: String,
}

impl PollTransactionParams {
    pub fn get_transaction_id(&self) -> String {
        self.transaction_id.clone().to_lowercase()
    }
}
