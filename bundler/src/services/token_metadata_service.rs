use sqlx::{Pool, Postgres};

use crate::db::dao::token_metadata_dao::TokenMetadataDao;
use crate::errors::errors::ApiError;
use crate::models::admin::metadata_response::MetadataResponse;
use crate::models::admin::metadata_response_v2::MetadataResponseV2;
use crate::CONFIG;

#[derive(Clone)]
pub struct TokenMetadataService;

impl TokenMetadataService {
    pub async fn get_chain(pool: &Pool<Postgres>) -> Result<MetadataResponse, ApiError> {
        let supported_currencies = TokenMetadataDao::get_metadata_by_currency(
            pool,
            CONFIG.run_config.current_chain.clone(),
            None,
        )
        .await
        .map_err(|_| ApiError::InternalServer(String::from("Failed to get data")))?;

        Ok(MetadataResponse::new().to(
            supported_currencies,
            CONFIG.run_config.current_chain.clone(),
            CONFIG.get_chain().chain_id,
            CONFIG.get_chain().currency.clone(),
        ))
    }

    pub async fn get_chain_v2(pool: &Pool<Postgres>) -> Result<MetadataResponseV2, ApiError> {
        Ok(MetadataResponseV2::from_token_metadata(
            TokenMetadataDao::get_metadata(pool)
                .await
                .map_err(|_| ApiError::InternalServer(String::from("Failed to fetch data")))?,
        ))
    }
}
