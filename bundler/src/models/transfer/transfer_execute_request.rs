use ethers::types::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TransferExecuteRequest {
    pub transaction_id: String,
    pub signature: Bytes,
}
