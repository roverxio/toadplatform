use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct AddMetadataRequest {
    pub chain: String,
    pub currency: String,
    pub contract_address: String,
    pub exponent: u8,
}

impl AddMetadataRequest {
    pub fn get_chain(&self) -> String {
        self.chain.to_lowercase()
    }

    pub fn get_currency(&self) -> String {
        self.currency.to_lowercase()
    }

    pub fn get_contract_address(&self) -> String {
        self.contract_address.to_lowercase()
    }

    pub fn get_exponent(&self) -> u8 {
        self.exponent
    }
}
