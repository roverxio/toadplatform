use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::Address;
use ethers::utils::format_ether;
use log::error;
use std::sync::Arc;

use crate::provider::web3_client::Web3Client;

abigen!(VerifyingPaymaster, "abi/VerifyingPaymaster.json");

pub struct VerifyingPaymasterProvider;

impl VerifyingPaymasterProvider {
    pub fn init_abi(
        address: Address,
        client: Arc<Provider<Http>>,
    ) -> VerifyingPaymaster<Provider<Http>> {
        let contract: VerifyingPaymaster<Provider<Http>> = VerifyingPaymaster::new(address, client);
        contract
    }

    pub async fn get_deposit(client: &Web3Client) -> Result<String, String> {
        let response = client
            .get_verifying_paymaster_provider()
            .get_deposit()
            .await;
        if response.is_err() {
            error!(
                "Paymaster: Deposit: {:?}",
                response.err().unwrap().to_string()
            );
            return Err(String::from("Failed to get balance"));
        }
        Ok(format_ether(response.unwrap()))
    }
}
