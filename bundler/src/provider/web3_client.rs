use crate::contracts::simple_account_factory_provider::{
    SimpleAccountFactory, SimpleAccountFactoryProvider,
};
use crate::contracts::usdc_provider::{USDCProvider, ERC20};
use crate::CONFIG;
use ethers::providers::{Http, Provider};
use std::sync::Arc;

#[derive(Clone)]
pub struct Web3Client {
    pub client: Arc<Provider<Http>>,
}

impl Web3Client {
    pub fn new(client: Arc<Provider<Http>>) -> Self {
        Self { client }
    }

    pub fn get_usdc_provider(&self) -> ERC20<Provider<Http>> {
        USDCProvider::init_abi(CONFIG.get_chain().usdc_address, self.client.clone())
    }

    pub fn get_factory_provider(&self) -> SimpleAccountFactory<Provider<Http>> {
        SimpleAccountFactoryProvider::init_abi(
            CONFIG.get_chain().simple_account_factory_address,
            self.client.clone(),
        )
    }
}
