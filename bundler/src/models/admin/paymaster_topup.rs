use crate::models::metadata::Metadata;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PaymasterTopup {
    pub value: String,
    pub metadata: Metadata,
}
