use ethers::abi::{Abi, Address};
use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::{Bytes, U256};
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

    pub async fn balance_of(abi: ERC20<Provider<Http>>, address: Address) -> Result<U256, String> {
        abi.balance_of(address)
            .await
            .map_err(|_| String::from("Failed to get balance"))
    }
}
