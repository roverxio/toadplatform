use ethers::middleware::SignerMiddleware;
use ethers::prelude::U256;
use ethers::providers::{Http, Middleware, Provider};
use ethers::types::Address;
use ethers_signers::{LocalWallet, Signer};
use log::error;
use std::sync::Arc;

use crate::contracts::entrypoint_provider::{EntryPoint, EntryPointProvider};
use crate::contracts::simple_account_factory_provider::{
    SimpleAccountFactory, SimpleAccountFactoryProvider,
};
use crate::contracts::simple_account_provider::{SimpleAccount, SimpleAccountProvider};
use crate::contracts::usdc_provider::{USDCProvider, ERC20};
use crate::contracts::verifying_paymaster_provider::{
    VerifyingPaymaster, VerifyingPaymasterProvider,
};
use crate::errors::ProviderError;
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

    pub fn get_verifying_paymaster_provider(&self) -> VerifyingPaymaster<Provider<Http>> {
        VerifyingPaymasterProvider::init_abi(
            CONFIG.get_chain().verifying_paymaster_address,
            self.client.clone(),
        )
    }

    pub fn get_entrypoint_provider(&self) -> EntryPoint<Provider<Http>> {
        EntryPointProvider::init_abi(CONFIG.get_chain().entrypoint_address, self.client.clone())
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

    pub fn get_relayer_wallet() -> LocalWallet {
        std::env::var("WALLET_PRIVATE_KEY")
            .expect("WALLET_PRIVATE_KEY must be set")
            .parse::<LocalWallet>()
            .unwrap()
    }

    pub fn get_verifying_paymaster_wallet() -> LocalWallet {
        std::env::var("VERIFYING_PAYMASTER_PRIVATE_KEY")
            .expect("VERIFYING_PAYMASTER_PRIVATE_KEY must be set")
            .parse::<LocalWallet>()
            .unwrap()
    }

    pub async fn estimate_eip1559_fees(&self) -> Result<(U256, U256), ProviderError> {
        let x = self.client.estimate_eip1559_fees(None).await;
        match x {
            Ok((gas_price, priority_fee)) => Ok((gas_price, priority_fee)),
            Err(_) => {
                error!("Failed to estimate eip1559 gas");
                Err(ProviderError("Failed to estimate eip1559 gas".to_string()))
            }
        }
    }
}
