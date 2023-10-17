use ethers::abi::Address;
use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::{Bytes, U256};
use std::sync::Arc;

use crate::errors::ProviderError;
use crate::provider::*;

abigen!(SimpleAccountFactory, "abi/SimpleAccountFactory.json");

#[derive(Clone)]
pub struct SimpleAccountFactoryProvider;

impl SimpleAccountFactoryProvider {
    pub fn init_abi(
        address: Address,
        client: Arc<Provider<Http>>,
    ) -> SimpleAccountFactory<Provider<Http>> {
        let contract: SimpleAccountFactory<Provider<Http>> =
            SimpleAccountFactory::new(address, client);
        contract
    }

    pub fn create_account(
        client: &Arc<Provider<Http>>,
        owner: Address,
        salt: U256,
    ) -> Result<Bytes, ProviderError> {
        let data = Web3Client::get_factory_provider(client.clone())
            .create_account(owner, salt)
            .calldata();
        match data {
            Some(call_data) => Ok(call_data),
            None => Err(ProviderError(String::from("create data failed"))),
        }
    }

    pub fn get_factory_address(client: Arc<Provider<Http>>) -> Address {
        Web3Client::get_factory_provider(client).address()
    }

    pub async fn get_address(
        client: Arc<Provider<Http>>,
        owner: Address,
        salt: u64,
    ) -> Result<Address, ProviderError> {
        let result = Web3Client::get_factory_provider(client)
            .get_address(owner, U256::from(salt))
            .await;
        match result {
            Ok(address) => Ok(address),
            Err(err) => Err(ProviderError(format!("Failed to get address: {:?}", err))),
        }
    }
}
