use crate::db::dao::metadata_dao::MetadataDao;
use crate::errors::ApiError;
use crate::models::metadata::Metadata;
use crate::CONFIG;

#[derive(Clone)]
pub struct MetadataService {
    pub metadata_dao: MetadataDao,
}

impl MetadataService {
    pub async fn get_chain(&self) -> Result<Metadata, ApiError> {
        let supported_currencies = self
            .metadata_dao
            .get_metadata_for_chain(CONFIG.run_config.current_chain.clone())
            .await;

        Ok(Metadata::new().to(
            supported_currencies,
            CONFIG.run_config.current_chain.clone(),
            CONFIG.get_chain().currency.clone(),
        ))
    }
}
