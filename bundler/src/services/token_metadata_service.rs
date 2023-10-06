use sqlx::{Pool, Postgres};

use crate::db::dao::TokenMetadataDao;
use crate::errors::MetadataError;
use crate::models::admin::{MetadataResponse, MetadataResponseV2};
use crate::CONFIG;

#[derive(Clone)]
pub struct TokenMetadataService;

impl TokenMetadataService {
    pub async fn get_chain(pool: &Pool<Postgres>) -> Result<MetadataResponse, MetadataError> {
        let supported_currencies = TokenMetadataDao::get_metadata_by_currency(
            pool,
            CONFIG.run_config.current_chain.clone(),
            None,
        )
        .await?;

        Ok(MetadataResponse::new().to(
            supported_currencies,
            CONFIG.run_config.current_chain.clone(),
            CONFIG.get_chain().chain_id,
            CONFIG.get_chain().currency.clone(),
        ))
    }

    pub async fn get_chain_v2(pool: &Pool<Postgres>) -> Result<MetadataResponseV2, MetadataError> {
        Ok(MetadataResponseV2::from_token_metadata(
            TokenMetadataDao::get_metadata(pool).await?,
        ))
    }
}
