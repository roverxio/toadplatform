use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_id: String,
    pub amount: Amount,
    pub metadata: Metadata,
    pub from: UserInfo,
    pub id: i32,
    pub to: UserInfo,
    #[serde(rename = "type")]
    pub transaction_type: String,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Amount {
    pub currency: String,
    pub value: String,
    pub exponent: i32,
}

#[derive(Serialize, Deserialize)]
pub struct UserInfo {
    pub address: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub chain: String,
    pub gas: Amount,
    pub transaction_hash: String,
    pub timestamp: String,
    pub explorer_url: String,
    pub status: String,
}
