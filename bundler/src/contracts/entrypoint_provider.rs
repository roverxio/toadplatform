use crate::CONFIG;
use ethers::abi::Abi;
use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::{Address, Bytes, U256};
use std::sync::Arc;

abigen!(EntryPoint, "abi/Entrypoint.json");

#[derive(Clone)]
pub struct EntryPointProvider {
    pub abi: EntryPoint<Provider<Http>>,
}

impl EntryPointProvider {
    pub fn abi(&self) -> &Abi {
        self.abi.abi()
    }

    pub fn init_abi(
        current_chain: &str,
        client: Arc<Provider<Http>>,
    ) -> EntryPoint<Provider<Http>> {
        let contract: EntryPoint<Provider<Http>> =
            EntryPoint::new(CONFIG.chains[current_chain].entrypoint_address, client);
        contract
    }
    pub async fn get_nonce(&self, sender: Address) -> Result<U256, String> {
        let result = self.abi.get_nonce(sender, U256::zero()).await;
        if result.is_err() {
            return Err(String::from("failed to get Nonce"));
        }
        Ok(result.unwrap())
    }

    pub async fn add_deposit(&self, address: Address) -> Result<Bytes, String> {
        let data = self.abi.deposit_to(address).calldata();
        if data.is_none() {
            return Err(String::from("add deposit data failed"));
        }
        Ok(data.unwrap())
    }
}
