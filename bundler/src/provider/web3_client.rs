use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Provider};
use ethers::types::Address;
use ethers_signers::{LocalWallet, Signer};
use std::sync::Arc;

use crate::contracts::simple_account_factory_provider::{
    SimpleAccountFactory, SimpleAccountFactoryProvider,
};
use crate::contracts::simple_account_provider::{SimpleAccount, SimpleAccountProvider};
use crate::contracts::usdc_provider::{USDCProvider, ERC20};
use crate::CONFIG;

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

    pub fn get_verifying_paymaster_provider(&self) -> SimpleAccount<Provider<Http>> {
        SimpleAccountProvider::init_abi(
            CONFIG.get_chain().verifying_paymaster_address,
            self.client.clone(),
        )
    }

    pub fn get_scw_provider_by_address(&self, address: Address) -> SimpleAccount<Provider<Http>> {
        SimpleAccountProvider::init_abi(address, self.client.clone())
    }

    pub fn get_relayer_signer(&self) -> SignerMiddleware<Arc<Provider<Http>>, LocalWallet> {
        SignerMiddleware::new(
            self.client.clone(),
            Self::get_relayer_wallet().with_chain_id(CONFIG.get_chain().chain_id),
        )
    }

    fn get_relayer_wallet() -> LocalWallet {
        std::env::var("WALLET_PRIVATE_KEY")
            .expect("WALLET_PRIVATE_KEY must be set")
            .parse::<LocalWallet>()
            .unwrap()
    }
}
