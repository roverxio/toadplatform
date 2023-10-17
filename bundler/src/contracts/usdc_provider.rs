use ethers::abi::Address;
use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::{Bytes, U256};
use log::error;
use std::sync::Arc;

use crate::errors::ProviderError;
use crate::provider::*;

abigen!(ERC20, "abi/ERC20.json");

#[derive(Clone)]
pub struct USDCProvider;

impl USDCProvider {
    pub fn init_abi(address: Address, client: Arc<Provider<Http>>) -> ERC20<Provider<Http>> {
        let contract: ERC20<Provider<Http>> = ERC20::new(address, client);
        contract
    }

    pub fn transfer(
        client: &Arc<Provider<Http>>,
        to: Address,
        value: String,
    ) -> Result<Bytes, ProviderError> {
        let data = Web3Client::get_usdc_provider(client.clone())
            .transfer(to, U256::from_dec_str(&value).unwrap())
            .calldata();
        match data {
            Some(call_data) => Ok(call_data),
            None => Err(ProviderError(String::from("transfer data failed"))),
        }
    }

    pub fn mint(client: &Arc<Provider<Http>>, to: Address, value: String) -> Result<Bytes, String> {
        let data = Web3Client::get_usdc_provider(client.clone())
            .sudo_mint(to, U256::from_dec_str(&value).unwrap())
            .calldata();
        if data.is_none() {
            return Err("mint data failed".to_string());
        }

        Ok(data.unwrap())
    }

    pub async fn balance_of(
        client: &Arc<Provider<Http>>,
        address: Address,
    ) -> Result<U256, ProviderError> {
        let result = Web3Client::get_usdc_provider(client.clone())
            .balance_of(address)
            .await;
        match result {
            Ok(balance) => Ok(balance),
            Err(err) => {
                error!("Failed to get balance: {}", err);
                Err(ProviderError(String::from("Failed to get balance")))
            }
        }
    }
}
