use crate::models::metadata::Metadata;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PaymasterTopup {
    pub address: String,
    pub value: String,
    pub metadata: Metadata,
}
