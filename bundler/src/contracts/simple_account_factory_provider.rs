use crate::provider::web3_client::Web3Client;
use ethers::abi::Address;
use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::{Bytes, U256};
use log::error;
use std::sync::Arc;

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

    pub fn create_account(&self, owner: Address, salt: U256) -> Result<Bytes, String> {
        let data = self.abi.create_account(owner, salt).calldata();
        if data.is_none() {
            return Err("create data failed".to_string());
        }

        Ok(data.unwrap())
    }

    pub async fn get_address(
        client: &Web3Client,
        owner: Address,
        salt: u64,
    ) -> Result<Address, String> {
        let result = client
            .get_factory_provider()
            .get_address(owner, U256::from(salt))
            .await;
        match result {
            Ok(address) => Ok(address),
            Err(err) => {
                error!("Failed to get address: {}", err);
                Err(String::from("Failed to get address"))
            }
        }
    }
}
