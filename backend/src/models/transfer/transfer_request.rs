use serde::Deserialize;
use crate::models::metadata::Metadata;

#[derive(Deserialize)]
pub struct TransferRequest {
    pub receiver: String,
    pub value: String,
    pub metadata: Metadata,
}
