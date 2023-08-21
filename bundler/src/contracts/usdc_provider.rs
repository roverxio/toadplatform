use crate::CONFIG;
use ethers::abi::Address;
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
    pub fn init_abi(current_chain: &str, client: Arc<Provider<Http>>) -> ERC20<Provider<Http>> {
        let contract: ERC20<Provider<Http>> =
            ERC20::new(CONFIG.chains[current_chain].usdc_address, client);
        contract
    }

    pub fn transfer(&self, to: Address, value: String) -> Result<Bytes, String> {
        let value: f64 = value.parse().unwrap();
        let usdc_amount = value * 1e6;
        let data = self
            .abi
            .transfer(to, U256::from(usdc_amount as u64))
            .calldata();
        if data.is_none() {
            return Err("transfer data failed".to_string());
        }

        Ok(data.unwrap())
    }
}
