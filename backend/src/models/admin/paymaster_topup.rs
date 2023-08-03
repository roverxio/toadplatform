use serde::Deserialize;
use crate::models::metadata::Metadata;

#[derive(Deserialize)]
pub struct PaymasterTopup {
    pub address: String,
    pub value: String,
    pub metadata: Metadata,
}
