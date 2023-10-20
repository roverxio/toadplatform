use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::{Address, Bytes, U256};
use std::sync::Arc;

use crate::errors::ProviderError;
use crate::provider::Web3Client;

abigen!(EntryPoint, "abi/Entrypoint.json");

#[derive(Clone)]
pub struct EntryPointProvider;

impl EntryPointProvider {
    pub fn init_abi(address: Address, client: Arc<Provider<Http>>) -> EntryPoint<Provider<Http>> {
        let contract: EntryPoint<Provider<Http>> = EntryPoint::new(address, client);
        contract
    }

    pub async fn get_nonce(client: &Web3Client, sender: Address) -> Result<U256, ProviderError> {
        let result = client
            .get_entrypoint_provider()
            .get_nonce(sender, U256::zero())
            .await;
        match result {
            Ok(nonce) => Ok(nonce),
            Err(err) => Err(ProviderError(format!("Failed to get Nonce: {:?}", err))),
        }
    }

    pub async fn add_deposit(
        client: &Web3Client,
        address: Address,
    ) -> Result<Bytes, ProviderError> {
        let data = client
            .get_entrypoint_provider()
            .deposit_to(address)
            .calldata();
        match data {
            Some(call_data) => Ok(call_data),
            None => Err(ProviderError(String::from("EP: Add deposit data failed"))),
        }
    }
}
