use crate::db::dao::token_metadata_dao::TokenMetadataDao;
use crate::errors::errors::ApiError;
use crate::models::admin::metadata_response::MetadataResponse;
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
}
