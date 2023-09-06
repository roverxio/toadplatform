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
        self.receiver.clone().to_lowercase()
    }

    pub fn get_value(&self) -> String {
        self.value.clone()
    }
}
