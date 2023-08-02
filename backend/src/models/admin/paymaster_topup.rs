use serde::Deserialize;

#[derive(Deserialize)]
pub struct Metadata {
    pub currency: String,
    pub chain: String,
}

#[derive(Deserialize)]
pub struct PaymasterTopup {
    pub address: String,
    pub value: String,
    pub metadata: Metadata,
}
