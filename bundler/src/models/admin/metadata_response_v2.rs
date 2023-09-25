use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::db::dao::token_metadata_dao::TokenMetadata;
use crate::CONFIG;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct MetadataResponseV2 {
    defaults: DefaultData,
    chains: HashMap<String, ChainDetail>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ChainDetail {
    chain_id: i32,
    tokens: HashMap<String, TokenDetail>,
    display_name: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct TokenDetail {
    image_url: String,
    display_name: String,
    exponent: i32,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DefaultData {
    chain: String,
    currency: String,
}

impl MetadataResponseV2 {
    pub fn from_token_metadata(tokens: Vec<TokenMetadata>) -> Self {
        let mut chains: HashMap<String, ChainDetail> = HashMap::new();

        for token in tokens {
            let token_detail = TokenDetail {
                image_url: token.token_image_url.unwrap_or_default(),
                display_name: token.name,
                exponent: token.exponent,
            };

            let chain_name = token.chain_name.unwrap_or(token.chain.clone());
            let chain_detail = chains.entry(token.chain.clone()).or_insert(ChainDetail {
                chain_id: token.chain_id.unwrap_or(0),
                tokens: HashMap::new(),
                display_name: chain_name.clone(),
            });

            chain_detail
                .tokens
                .insert(token.symbol.clone(), token_detail);
        }

        let defaults = DefaultData {
            chain: CONFIG.run_config.current_chain.clone(),
            currency: CONFIG.run_config.default_currency.clone(),
        };

        MetadataResponseV2 { defaults, chains }
    }
}
