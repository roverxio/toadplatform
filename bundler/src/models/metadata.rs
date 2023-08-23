use crate::db::dao::metadata_dao::SupportedCurrency;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug)]
pub struct Metadata {
    pub currency: String,
    pub chain: String,
    pub exponents: HashMap<String, u8>,
}
impl Metadata {
    pub fn new() -> Metadata {
        Metadata {
            currency: "".to_string(),
            chain: "".to_string(),
            exponents: HashMap::new(),
        }
    }
    pub fn to(
        mut self,
        metadata: Vec<SupportedCurrency>,
        chain: String,
        currency: String,
    ) -> Metadata {
        let mut exponents: HashMap<String, u8> = HashMap::new();

        for item in metadata {
            exponents.insert(item.currency, item.exponent);
        }
        self.chain = chain;
        self.currency = currency;
        self.exponents = exponents;

        self
    }
}
