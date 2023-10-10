use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::db::dao::TokenMetadata;

#[derive(Default, Deserialize, Serialize, Debug)]
pub struct MetadataResponse {
    currency: String,
    chain: String,
    chain_id: u64,
    exponents: HashMap<String, i32>,
    tokens: HashMap<String, TokenMetadataResponse>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenMetadataResponse {
    name: String,
    exponent: i32,
}

impl MetadataResponse {
    pub fn new() -> MetadataResponse {
        MetadataResponse::default()
    }
    pub fn to(
        mut self,
        token_metadata: Vec<TokenMetadata>,
        chain: String,
        chain_id: u64,
        currency: String,
    ) -> MetadataResponse {
        let mut exponents: HashMap<String, i32> = HashMap::new();
        let mut metadata: HashMap<String, TokenMetadataResponse> = HashMap::new();

        for item in token_metadata {
            exponents.insert(item.symbol.clone(), item.exponent);
            metadata.insert(
                item.symbol,
                TokenMetadataResponse {
                    name: item.name,
                    exponent: item.exponent,
                },
            );
        }
        self.chain = chain;
        self.chain_id = chain_id;
        self.currency = currency;
        self.exponents = exponents;
        self.tokens = metadata;

        self
    }
}
