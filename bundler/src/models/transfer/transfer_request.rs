use crate::models::metadata::Metadata;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TransferRequest {
    pub receiver: String,
    pub value: String,
    pub metadata: Metadata,
}

impl TransferRequest {
    pub fn get_receiver(&self) -> String {
        self.receiver.trim().trim().to_string()
    }

    pub fn get_value(&self) -> String {
        self.value.trim().to_string()
    }
}
