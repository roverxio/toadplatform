use ethers::abi::Address;
use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::{Bytes, U256};
use log::error;
use std::sync::Arc;

use crate::provider::web3_client::Web3Client;

abigen!(SimpleAccount, "abi/SimpleAccount.json");

#[derive(Clone)]
pub struct SimpleAccountProvider {
    pub abi: SimpleAccount<Provider<Http>>,
}

impl SimpleAccountProvider {
    pub fn init_abi(
        address: Address,
        client: Arc<Provider<Http>>,
    ) -> SimpleAccount<Provider<Http>> {
        let contract: SimpleAccount<Provider<Http>> = SimpleAccount::new(address, client);
        contract
    }

    pub fn execute(&self, to: Address, value: String, data: Bytes) -> Result<Bytes, String> {
        let data = self
            .abi
            .execute(to, U256::from_dec_str(&value).unwrap(), data)
            .calldata();
        if data.is_none() {
            return Err("execute data failed".to_string());
        }

        Ok(data.unwrap())
    }

    pub async fn get_deployer(
        client: &Web3Client,
        contract_address: Address,
    ) -> Result<String, String> {
        let result = client
            .get_scw_provider_by_address(contract_address)
            .deployed_by()
            .call()
            .await;
        match result {
            Ok(address) => Ok(address),
            Err(err) => {
                error!("Failed to get deployer: {}", err);
                Err(String::from("Failed to get deployer"))
            }
        }
    }
}
