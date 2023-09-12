use ethers::types::Bytes;
use serde::Serialize;

#[derive(Serialize)]
pub struct TransferInitResponse {
    pub msg_hash: Bytes,
    pub status: String,
    pub transaction_id: String,
}
