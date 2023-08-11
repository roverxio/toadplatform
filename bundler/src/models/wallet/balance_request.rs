use base64::engine::general_purpose;
use base64::Engine;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BalanceRequest {
    pub q: String,
}

#[derive(Deserialize)]
pub struct Balance {
    pub chain: String,
    pub currency: String,
}

impl BalanceRequest {
    pub fn get_balance_request(&self) -> Balance {
        let bytes = general_purpose::STANDARD.decode(&self.q).unwrap();
        serde_json::from_slice(&bytes).expect("JSON deserialization failed!")
    }
}
