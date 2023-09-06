use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct AddMetadataRequest {
    pub chain: String,
    pub currency: String,
    pub contract_address: String,
    pub exponent: i32,
    pub token_type: String,
    pub name: String,
}

impl AddMetadataRequest {
    pub fn get_chain(&self) -> String {
        self.chain.trim().to_lowercase()
    }

    pub fn get_currency(&self) -> String {
        self.currency.trim().to_lowercase()
    }

    pub fn get_contract_address(&self) -> String {
        self.contract_address.trim().to_lowercase()
    }

    pub fn get_exponent(&self) -> i32 {
        self.exponent
    }

    pub fn get_token_type(&self) -> String {
        self.token_type.trim().to_lowercase()
    }

    pub fn get_name(&self) -> String {
        self.name.trim().to_lowercase()
    }
}
