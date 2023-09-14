use serde::Serialize;

#[derive(Serialize)]
pub struct TransferInitResponse {
    pub msg_hash: [u8; 32],
    pub status: String,
    pub transaction_id: String,
}
