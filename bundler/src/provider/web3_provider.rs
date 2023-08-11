use std::sync::Arc;

use ethers::prelude::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::Address;

use crate::CONFIG;

pub struct Web3Provider {}

abigen!(SimpleAccountFactory, "abi/SimpleAccountFactory.json");
abigen!(ERC20, "abi/ERC20.json");
abigen!(Simpleaccount, "abi/SimpleAccount.json");

impl Web3Provider {
    pub fn new(chain_url: String) -> Provider<Http> {
        let provider = Provider::try_from(chain_url).unwrap();
        provider
    }

    pub fn get_simple_account_factory_abi(
        current_chain: &str,
        client: Arc<Provider<Http>>,
    ) -> SimpleAccountFactory<Provider<Http>> {
        let contract: SimpleAccountFactory<Provider<Http>> = SimpleAccountFactory::new(
            CONFIG.chains[current_chain].simple_account_factory_address,
            client,
        );
        contract
    }

    pub fn get_erc20_abi(
        current_chain: &str,
        client: Arc<Provider<Http>>,
    ) -> ERC20<Provider<Http>> {
        let contract: ERC20<Provider<Http>> =
            ERC20::new(CONFIG.chains[current_chain].usdc_address, client);
        contract
    }

    pub fn get_simpleaccount_abi(client: Arc<Provider<Http>>) -> Simpleaccount<Provider<Http>> {
        let contract: Simpleaccount<Provider<Http>> = Simpleaccount::new(Address::zero(), client);
        contract
    }
}
