use crate::contracts::usdc_provider::{USDCProvider, ERC20};
use crate::CONFIG;
use ethers::providers::{Http, Provider};
use std::sync::Arc;

#[derive(Clone)]
pub struct Web3Client {
    pub client: Arc<Provider<Http>>,
    pub usdc_provider: Option<ERC20<Provider<Http>>>,
}

impl Web3Client {
    pub fn new(client: Arc<Provider<Http>>) -> Self {
        Self {
            client,
            usdc_provider: None,
        }
    }

    pub fn get_usdc_provider(&self) -> ERC20<Provider<Http>> {
        USDCProvider::init_abi(CONFIG.get_chain().usdc_address, self.client.clone())
    }
}
