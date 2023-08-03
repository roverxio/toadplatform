use serde::Deserialize;

#[derive(Deserialize)]
pub struct TransferRequest {
    pub receiver: String,
    pub value: String,
    pub metadata: Metadata,
}

#[derive(Deserialize)]
pub struct Metadata {
    pub currency: String,
    pub chain: String,
}

