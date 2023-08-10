use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Metadata {
    pub currency: String,
    pub chain: String,
}
