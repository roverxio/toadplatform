use crate::errors::base::ProviderError;
use crate::provider::web3_client::Web3Client;
use ethers::abi::{Abi, Address};
use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::{Bytes, U256};
use log::error;
use std::sync::Arc;

abigen!(ERC20, "abi/ERC20.json");

#[derive(Clone)]
pub struct USDCProvider {
    pub abi: ERC20<Provider<Http>>,
}

impl USDCProvider {
    pub fn abi(&self) -> &Abi {
        self.abi.abi()
    }

    pub fn init_abi(address: Address, client: Arc<Provider<Http>>) -> ERC20<Provider<Http>> {
        let contract: ERC20<Provider<Http>> = ERC20::new(address, client);
        contract
    }

    pub fn transfer(&self, to: Address, value: String) -> Result<Bytes, String> {
        let data = self
            .abi
            .transfer(to, U256::from_dec_str(&value).unwrap())
            .calldata();
        if data.is_none() {
            return Err("transfer data failed".to_string());
        }

        Ok(data.unwrap())
    }

    pub fn mint(&self, to: Address, value: String) -> Result<Bytes, String> {
        let data = self
            .abi
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
    use crate::contracts::usdc_provider::USDCProvider;
    use crate::provider::web3_client::Web3Client;
    use crate::PROVIDER;
    use std::sync::Arc;

    #[actix_web::test]
    async fn test_get_balance_success() {
        let web3_client = Web3Client::new(Arc::new(PROVIDER.clone()));
        let address = "0x1bb719eec37efff15ab534f5ea24107531f58bfa"
            .parse()
            .unwrap();

        let result = USDCProvider::balance_of(&web3_client, address).await;

        assert!(result.is_ok());
    }
}
