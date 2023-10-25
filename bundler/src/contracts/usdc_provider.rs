use ethers::abi::Address;
use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::{Bytes, U256};
use log::error;
use std::sync::Arc;

use crate::errors::ProviderError;
use crate::provider::Web3Client;

abigen!(ERC20, "abi/ERC20.json");

#[derive(Clone)]
pub struct USDCProvider;

#[mockall::automock]
impl USDCProvider {
    pub fn init_abi(address: Address, client: Arc<Provider<Http>>) -> ERC20<Provider<Http>> {
        let contract: ERC20<Provider<Http>> = ERC20::new(address, client);
        contract
    }

    pub fn transfer(
        client: &Web3Client,
        to: Address,
        value: String,
    ) -> Result<Bytes, ProviderError> {
        let data = client
            .get_usdc_provider()
            .transfer(to, U256::from_dec_str(&value).unwrap())
            .calldata();
        match data {
            Some(call_data) => Ok(call_data),
            None => Err(ProviderError(String::from("transfer data failed"))),
        }
    }

    pub fn mint(client: &Web3Client, to: Address, value: String) -> Result<Bytes, String> {
        let data = client
            .get_usdc_provider()
            .sudo_mint(to, U256::from_dec_str(&value).unwrap())
            .calldata();
        if data.is_none() {
            return Err("mint data failed".to_string());
        }

        Ok(data.unwrap())
    }

    pub async fn balance_of(client: &Web3Client, address: Address) -> Result<U256, ProviderError> {
        let result = client.get_usdc_provider().balance_of(address).await;
        match result {
            Ok(balance) => Ok(balance),
            Err(err) => {
                error!("Failed to get balance: {}", err);
                Err(ProviderError(String::from("Failed to get balance")))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::provider::web3_client::tests::setup_mock_client;
    use crate::provider::web3_provider::tests::setup_mock_provider;

    #[tokio::test]
    async fn test_init_abi() {
        let provider = setup_mock_provider();
        let mock_init = MockUSDCProvider::init_abi_context();

        mock_init.expect().returning(|address, client| {
            let contract: ERC20<Provider<Http>> = ERC20::new(address, client);
            return contract;
        });

        let abi = MockUSDCProvider::init_abi(Address::zero(), Arc::new(provider.clone()));

        assert_eq!(
            abi.address(),
            ERC20::new(Address::zero(), Arc::new(provider)).address()
        )
    }

    #[tokio::test]
    async fn test_balance_of_success() {
        let web3_client = setup_mock_client();

        let mock_balance = MockUSDCProvider::balance_of_context();
        mock_balance.expect().returning(|_, _| Ok(U256::zero()));

        let result = MockUSDCProvider::balance_of(&web3_client, Address::zero()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), U256::zero());
    }

    #[tokio::test]
    async fn test_balance_of_failure() {
        let web3_client = setup_mock_client();

        let mock_balance = MockUSDCProvider::balance_of_context();
        mock_balance
            .expect()
            .returning(|_, _| Err(ProviderError("failed to get balance".to_string())));

        let result = MockUSDCProvider::balance_of(&web3_client, Address::zero()).await;

        assert!(result.is_err());
    }
}
