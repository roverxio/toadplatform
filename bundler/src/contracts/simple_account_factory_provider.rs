use ethers::abi::Address;
use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::{Bytes, U256};
use std::sync::Arc;

use crate::errors::ProviderError;
use crate::provider::Web3Client;

abigen!(SimpleAccountFactory, "abi/SimpleAccountFactory.json");

#[derive(Clone)]
pub struct SimpleAccountFactoryProvider {
    pub abi: SimpleAccountFactory<Provider<Http>>,
}

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
        abi: SimpleAccountFactory<Provider<Http>>,
        owner: Address,
        salt: U256,
    ) -> Result<Bytes, String> {
        let data = abi.create_account(owner, salt).calldata();
        match data {
            Some(call_data) => Ok(call_data),
            None => Err(String::from("create data failed")),
        }
    }

    pub fn get_factory_address(abi: SimpleAccountFactory<Provider<Http>>) -> Address {
        abi.address()
    }

    pub async fn get_address(
        client: &Web3Client,
        owner: Address,
        salt: u64,
    ) -> Result<Address, ProviderError> {
        let result = client
            .get_factory_provider()
            .get_address(owner, U256::from(salt))
            .await;
        match result {
            Ok(address) => Ok(address),
            Err(err) => Err(ProviderError(format!("Failed to get address: {:?}", err))),
        }
    }
}
