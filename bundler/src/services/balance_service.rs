use ethers::abi::Address;
use ethers::providers::{Http, Middleware, Provider};
use log::info;

use crate::contracts::usdc_provider::ERC20;
use crate::db::dao::token_metadata_dao::TokenMetadataDao;
use crate::db::dao::wallet_dao::{User, WalletDao};
use crate::errors::ApiError;
use crate::models::currency::Currency;
use crate::models::wallet::balance_response::BalanceResponse;
use crate::PROVIDER;

#[derive(Clone)]
pub struct BalanceService {
    pub wallet_dao: WalletDao,
    pub token_metadata_dao: TokenMetadataDao,
    pub erc20_provider: ERC20<Provider<Http>>,
}

impl BalanceService {
    pub async fn get_wallet_balance(
        &self,
        chain: &String,
        currency: &String,
        user: User,
    ) -> Result<BalanceResponse, ApiError> {
        info!("Chain: {:?}", chain); // will be relevant when we add support for multiple chains
        let balance: String;
        if user.wallet_address.is_empty() {
            return Err(ApiError::NotFound("Wallet not found".to_string()));
        }
        let wallet_address: Address = user.wallet_address.parse().unwrap();

        match Currency::from_str(
            self.token_metadata_dao
                .get_metadata_for_chain(chain.clone(), Some(currency.to_string()))
                .await[0]
                .token_type
                .clone(),
        ) {
            None => return Err(ApiError::BadRequest("Currency not supported".to_string())),
            Some(Currency::Erc20) => {
                balance = self
                    .erc20_provider
                    .balance_of(wallet_address.clone())
                    .await
                    .unwrap()
                    .to_string();
            }
            Some(Currency::Native) => {
                balance = PROVIDER
                    .get_balance(wallet_address.clone(), None)
                    .await
                    .unwrap()
                    .to_string();
            }
        }

        Ok(BalanceResponse {
            balance: balance.clone(),
            address: user.wallet_address,
            currency: currency.to_string(),
            exponent: self
                .token_metadata_dao
                .get_metadata_for_chain(chain.clone(), Some(currency.clone()))
                .await[0]
                .exponent,
        })
    }
}
