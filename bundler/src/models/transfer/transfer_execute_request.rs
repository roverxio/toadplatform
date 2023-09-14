use ethers::types::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TransferExecuteRequest {
    pub transaction_id: String,
    pub signature: Vec<u8>,
}

impl TransferExecuteRequest {
    pub fn get_signature(&self) -> Bytes {
        Bytes::from(self.signature.clone())
    }
}
