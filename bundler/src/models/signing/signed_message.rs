use ethers::types::Bytes;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SignedPayload {
    pub sign: Bytes,
}
