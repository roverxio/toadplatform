use ethers::abi::Address;
use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use std::sync::Arc;

abigen!(SimpleAccount, "abi/SimpleAccount.json");

#[derive(Clone)]
pub struct SimpleAccountProvider {
    pub abi: SimpleAccount<Provider<Http>>,
}

impl SimpleAccountProvider {
    pub fn init_abi(
        client: Arc<Provider<Http>>,
        address: Address,
    ) -> SimpleAccount<Provider<Http>> {
        let contract: SimpleAccount<Provider<Http>> = SimpleAccount::new(address, client);
        contract
    }
}
