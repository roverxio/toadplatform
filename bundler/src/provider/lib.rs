use ethers::types::U256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Request<T> {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: T,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EstimateResult {
    pub pre_verification_gas: U256,
    pub verification_gas_limit: U256,
    pub call_gas_limit: U256,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<T>,
    pub error: Option<ErrorDetails>,
}
