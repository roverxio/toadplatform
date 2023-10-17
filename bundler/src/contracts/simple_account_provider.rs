use ethers::abi::Address;
use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::{Bytes, U256};
use std::sync::Arc;

use crate::errors::ProviderError;
use crate::provider::*;

abigen!(SimpleAccount, "abi/SimpleAccount.json");

#[derive(Clone)]
pub struct SimpleAccountProvider;

impl SimpleAccountProvider {
    pub fn init_abi(
        address: Address,
        client: Arc<Provider<Http>>,
    ) -> SimpleAccount<Provider<Http>> {
        let contract: SimpleAccount<Provider<Http>> = SimpleAccount::new(address, client);
        contract
    }

    pub fn execute(
        client: &Arc<Provider<Http>>,
        to: Address,
        value: String,
        data: Bytes,
    ) -> Result<Bytes, ProviderError> {
        let data = Web3Client::get_scw_provider_by_address(client.clone(), Address::zero())
            .execute(to, U256::from_dec_str(&value).unwrap(), data)
            .calldata();
        match data {
            Some(call_data) => Ok(call_data),
            None => Err(ProviderError(String::from("execute data failed"))),
        }
    }

    pub async fn get_deployer(
        client: &Arc<Provider<Http>>,
        contract_address: Address,
    ) -> Result<String, ProviderError> {
        let result = Web3Client::get_scw_provider_by_address(client.clone(), contract_address)
            .deployed_by()
            .call()
            .await;
        match result {
            Ok(address) => Ok(address),
            Err(err) => Err(ProviderError(format!("Failed to get deployer: {}", err))),
        }
    }
}
