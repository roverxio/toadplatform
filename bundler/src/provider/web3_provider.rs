use std::sync::Arc;

use ethers::prelude::abigen;
use ethers::providers::{Http, Middleware, Provider};
use ethers::types::Address;
use log::error;

use crate::{CONFIG, PROVIDER};
use crate::errors::ApiError;

pub struct Web3Provider {}

abigen!(SimpleAccountFactory, "abi/SimpleAccountFactory.json");
abigen!(ERC20, "abi/ERC20.json");
abigen!(Simpleaccount, "abi/SimpleAccount.json");

impl Web3Provider {
    pub fn new(chain_url: String) -> Provider<Http> {
        let provider = Provider::try_from(chain_url).unwrap();
        provider
    }

    pub fn get_simple_account_factory_abi(
        current_chain: &str,
        client: Arc<Provider<Http>>,
    ) -> SimpleAccountFactory<Provider<Http>> {
        let contract: SimpleAccountFactory<Provider<Http>> = SimpleAccountFactory::new(
            CONFIG.chains[current_chain].simple_account_factory_address,
            client,
        );
        contract
    }

    pub fn get_erc20_abi(
        current_chain: &str,
        client: Arc<Provider<Http>>,
    ) -> ERC20<Provider<Http>> {
        let contract: ERC20<Provider<Http>> =
            ERC20::new(CONFIG.chains[current_chain].usdc_address, client);
        contract
    }

    pub fn get_simpleaccount_abi(client: Arc<Provider<Http>>) -> Simpleaccount<Provider<Http>> {
        let contract: Simpleaccount<Provider<Http>> = Simpleaccount::new(Address::zero(), client);
        contract
    }

    pub async fn get_native_balance(address: Address) -> Result<String, ApiError> {
        let result = PROVIDER
            .get_balance(address, None)
            .await;
        if result.is_err() {
            error!("Get native balance failed: {:?}", result.err().unwrap());
            return Err(ApiError::InternalServer("Failed to get balance".to_string()));
        }
        let wei_balance = result.unwrap().to_string();
        if wei_balance.parse::<f64>().is_err() {
            return Err(ApiError::InternalServer("Failed to parse balance".to_string()));
        }
        Ok((wei_balance.parse::<f64>().unwrap() / 1e18).to_string())
    }
}
