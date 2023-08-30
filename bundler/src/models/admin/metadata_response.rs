use crate::db::dao::metadata_dao::SupportedCurrency;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Deserialize, Serialize, Debug)]
pub struct MetadataResponse {
    pub currency: String,
    pub chain: String,
    pub exponents: HashMap<String, i32>,
}
impl MetadataResponse {
    pub fn new() -> MetadataResponse {
        MetadataResponse::default()
    }
    pub fn to(
        mut self,
        metadata: Vec<SupportedCurrency>,
        chain: String,
        currency: String,
    ) -> MetadataResponse {
        let mut exponents: HashMap<String, i32> = HashMap::new();

        for item in metadata {
            exponents.insert(item.currency, item.exponent);
        }
        self.chain = chain;
        self.currency = currency;
        self.exponents = exponents;

        self
    }
}
