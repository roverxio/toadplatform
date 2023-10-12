use serde::Deserialize;

use crate::models::Metadata;

#[derive(Deserialize)]
pub struct PaymasterTopup {
    pub value: String,
    pub metadata: Metadata,
}
