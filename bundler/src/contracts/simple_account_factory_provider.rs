use crate::CONFIG;
use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::{Address, Bytes, U256};
use std::sync::Arc;

abigen!(SimpleAccountFactory, "abi/SimpleAccountFactory.json");

#[derive(Clone)]
pub struct SimpleAccountFactoryProvider {
    pub abi: SimpleAccountFactory<Provider<Http>>,
}

impl SimpleAccountFactoryProvider {
    pub fn init_abi(
        current_chain: &str,
        client: Arc<Provider<Http>>,
    ) -> SimpleAccountFactory<Provider<Http>> {
        let contract: SimpleAccountFactory<Provider<Http>> = SimpleAccountFactory::new(
            CONFIG.chains[current_chain].simple_account_factory_address,
            client,
        );
        contract
    }

    pub fn create_account(&self, owner: Address, salt: U256) -> Result<Bytes, String> {
        let data = self.abi.create_account(owner, salt).calldata();
        if data.is_none() {
            return Err("create data failed".to_string());
        }

        Ok(data.unwrap())
    }
}
