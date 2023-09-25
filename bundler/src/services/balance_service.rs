use ethers::abi::Address;
use ethers::providers::Middleware;
use ethers::types::U256;
use log::{error, info};
use sqlx::{Pool, Postgres};

use crate::contracts::usdc_provider::USDCProvider;
use crate::db::dao::token_metadata_dao::TokenMetadataDao;
use crate::db::dao::wallet_dao::User;
use crate::errors::balance::BalanceError;
use crate::models::currency::Currency;
use crate::models::wallet::balance_response::BalanceResponse;
use crate::provider::web3_client::Web3Client;
use crate::PROVIDER;

#[derive(Clone)]
pub struct BalanceService;

impl BalanceService {
    pub async fn get_wallet_balance(
        pool: &Pool<Postgres>,
        provider: &Web3Client,
        chain: &String,
        currency: &String,
        user: User,
    ) -> Result<BalanceResponse, BalanceError> {
        info!("Chain: {:?}", chain); // will be relevant when we add support for multiple chains
        let balance: U256;
        if user.wallet_address.is_empty() {
            return Err(BalanceError::NotFound);
        }
        let wallet_address: Address = user.wallet_address.parse().unwrap();
        let metadata =
            TokenMetadataDao::get_metadata(pool, chain.clone(), Some(currency.clone())).await?;
        if metadata.is_empty() {
            return Err(BalanceError::InvalidCurrency);
        }

        match Currency::from_str(metadata[0].token_type.clone()) {
            None => return Err(BalanceError::InvalidCurrency),
            Some(Currency::Erc20) => {
                balance = USDCProvider::balance_of(provider, wallet_address.clone()).await?;
            }
            Some(Currency::Native) => {
                balance = PROVIDER
                    .get_balance(wallet_address.clone(), None)
                    .await
                    .map_err(|error| {
                        error!("Web3 Provider Error: {error}");
                        BalanceError::Provider(String::from("Failed to get balance"))
                    })?;
            }
        }

        Ok(BalanceResponse {
            balance: balance.to_string(),
            address: user.wallet_address,
            currency: currency.to_string(),
            exponent: metadata[0].exponent,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::db::connection::DatabaseConnection;
    use crate::db::dao::wallet_dao::User;
    use crate::errors::balance::BalanceError;
    use crate::provider::web3_client::Web3Client;
    use crate::services::balance_service::BalanceService;
    use crate::PROVIDER;
    use std::sync::Arc;

    #[actix_web::test]
    async fn test_get_balance_with_default_user() {
        let pool = DatabaseConnection::init().await;
        let chain = String::from("localhost");
        let currency = String::from("USDC");
        let web3_client = Web3Client::new(Arc::new(PROVIDER.clone()));
        let user: User = Default::default();

        let result =
            BalanceService::get_wallet_balance(&pool, &web3_client, &chain, &currency, user).await;

        assert_eq!(result.err().unwrap(), BalanceError::NotFound);
    }

    #[actix_web::test]
    async fn test_get_balance_with_invalid_currency() {
        let pool = DatabaseConnection::init().await;
        let chain = String::from("localhost");
        let currency = String::from("");
        let web3_client = Web3Client::new(Arc::new(PROVIDER.clone()));
        let mut user: User = Default::default();
        user.wallet_address = "0x1bb719eec37efff15ab534f5ea24107531f58bfa".to_string();

        let result =
            BalanceService::get_wallet_balance(&pool, &web3_client, &chain, &currency, user).await;

        assert_eq!(result.err().unwrap(), BalanceError::InvalidCurrency);
    }

    #[actix_web::test]
    async fn test_get_balance_success() {
        let pool = DatabaseConnection::init().await;
        let chain = String::from("localhost");
        let currency = String::from("USDC");
        let web3_client = Web3Client::new(Arc::new(PROVIDER.clone()));
        let mut user: User = Default::default();
        user.wallet_address = "0x1bb719eec37efff15ab534f5ea24107531f58bfa".to_string();

        let result =
            BalanceService::get_wallet_balance(&pool, &web3_client, &chain, &currency, user).await;

        assert_eq!(result.is_ok(), true);
    }
}
