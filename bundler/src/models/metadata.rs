use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Metadata {
    pub currency: String,
    pub chain: String,
}

impl Metadata {
    pub fn get_currency(&self) -> String {
        self.currency.clone().to_lowercase()
    }
}
