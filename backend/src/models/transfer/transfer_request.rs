use crate::models::metadata::Metadata;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TransferRequest {
    pub receiver: String,
    pub value: String,
    pub metadata: Metadata,
}
