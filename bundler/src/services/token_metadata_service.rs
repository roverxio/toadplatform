use crate::db::dao::token_metadata_dao::TokenMetadataDao;
use crate::errors::errors::ApiError;
use crate::models::admin::{MetadataResponse, MetadataResponseV2};
use crate::CONFIG;

#[derive(Clone)]
pub struct TokenMetadataService {
    pub token_metadata_dao: TokenMetadataDao,
}

impl TokenMetadataService {
    pub async fn get_chain(&self) -> Result<MetadataResponse, ApiError> {
        let supported_currencies = self
            .token_metadata_dao
            .get_metadata_for_chain(CONFIG.run_config.current_chain.clone(), None)
            .await;

        Ok(MetadataResponse::new().to(
            supported_currencies,
            CONFIG.run_config.current_chain.clone(),
            CONFIG.get_chain().chain_id,
            CONFIG.get_chain().currency.clone(),
        ))
    }

    pub async fn get_chain_v2(&self) -> Result<MetadataResponseV2, ApiError> {
        Ok(MetadataResponseV2::from_token_metadata(
            self.token_metadata_dao.get_metadata().await,
        ))
    }
}
