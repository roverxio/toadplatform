use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AddMetadataRequest {
    chain: Chain,
    token: Token,
}

#[derive(Serialize, Deserialize)]
pub struct Chain {
    name: String,
    id: i32,
    display_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Token {
    symbol: String,
    image_url: String,
    contract_address: String,
    #[serde(rename = "type")]
    token_type: String,
    display_name: String,
    exponent: i32,
}

impl AddMetadataRequest {
    pub fn get_chain_name(&self) -> String {
        self.chain.name.trim().to_string()
    }

    pub fn get_symbol(&self) -> String {
        self.token.symbol.trim().to_string()
    }

    pub fn get_contract_address(&self) -> String {
        self.token.contract_address.trim().to_lowercase()
    }

    pub fn get_exponent(&self) -> i32 {
        self.token.exponent
    }

    pub fn get_token_type(&self) -> String {
        self.token.token_type.trim().to_lowercase()
    }

    pub fn get_token_name(&self) -> String {
        self.token.display_name.trim().to_string()
    }

    pub fn get_chain_id(&self) -> i32 {
        self.chain.id
    }

    pub fn get_chain_display_name(&self) -> String {
        self.chain.display_name.trim().to_string()
    }

    pub fn get_token_image_url(&self) -> String {
        self.token.image_url.trim().to_string()
    }
}
