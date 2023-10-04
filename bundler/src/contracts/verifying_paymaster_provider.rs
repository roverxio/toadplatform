use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::Address;
use std::sync::Arc;

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
}
