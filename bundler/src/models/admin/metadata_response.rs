use crate::db::dao::token_metadata_dao::TokenMetadata;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Deserialize, Serialize, Debug)]
pub struct MetadataResponse {
    currency: String,
    chain: String,
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
        self.currency = currency;
        self.exponents = exponents;
        self.tokens = metadata;

        self
    }
}
