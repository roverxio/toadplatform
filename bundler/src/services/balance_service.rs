use ethers::abi::Address;
use ethers::providers::Middleware;
use ethers::types::U256;
use log::{error, info};
use sqlx::{Pool, Postgres};

use crate::contracts::usdc_provider::USDCProvider;
use crate::db::dao::token_metadata_dao::TokenMetadataDao;
use crate::db::dao::wallet_dao::User;
use crate::errors::BalanceError;
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
            TokenMetadataDao::get_metadata_by_currency(pool, chain.clone(), Some(currency.clone()))
                .await?;
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
