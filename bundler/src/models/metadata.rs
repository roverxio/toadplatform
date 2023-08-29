use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Metadata {
    pub currency: String,
    pub chain: String,
}
