use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Provider};
use ethers::types::Address;
use ethers_signers::{LocalWallet, Signer};
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
use crate::CONFIG;

pub trait Client {
    fn get_usdc_provider(client: Arc<Provider<Http>>) -> ERC20<Provider<Http>> {
        USDCProvider::init_abi(CONFIG.get_chain().usdc_address, client)
    }

    fn get_factory_provider(client: Arc<Provider<Http>>) -> SimpleAccountFactory<Provider<Http>> {
        SimpleAccountFactoryProvider::init_abi(
            CONFIG.get_chain().simple_account_factory_address,
            client,
        )
    }

    fn get_verifying_paymaster_provider(
        client: Arc<Provider<Http>>,
    ) -> VerifyingPaymaster<Provider<Http>> {
        VerifyingPaymasterProvider::init_abi(CONFIG.get_chain().verifying_paymaster_address, client)
    }

    fn get_entrypoint_provider(client: Arc<Provider<Http>>) -> EntryPoint<Provider<Http>> {
        EntryPointProvider::init_abi(CONFIG.get_chain().entrypoint_address, client)
    }

    fn get_scw_provider_by_address(
        client: Arc<Provider<Http>>,
        address: Address,
    ) -> SimpleAccount<Provider<Http>> {
        SimpleAccountProvider::init_abi(address, client)
    }

    fn get_relayer_signer(
        client: Arc<Provider<Http>>,
    ) -> SignerMiddleware<Arc<Provider<Http>>, LocalWallet> {
        SignerMiddleware::new(
            client,
            Self::get_relayer_wallet().with_chain_id(CONFIG.get_chain().chain_id),
        )
    }

    fn get_bundler_signer(
        client: Arc<Provider<Http>>,
    ) -> SignerMiddleware<Arc<Provider<Http>>, LocalWallet> {
        SignerMiddleware::new(
            client,
            Self::get_relayer_wallet().with_chain_id(CONFIG.get_chain().chain_id),
        )
    }

    fn get_relayer_wallet() -> LocalWallet {
        std::env::var("WALLET_PRIVATE_KEY")
            .expect("WALLET_PRIVATE_KEY must be set")
            .parse::<LocalWallet>()
            .unwrap()
    }

    fn get_verifying_paymaster_wallet() -> LocalWallet {
        std::env::var("VERIFYING_PAYMASTER_PRIVATE_KEY")
            .expect("VERIFYING_PAYMASTER_PRIVATE_KEY must be set")
            .parse::<LocalWallet>()
            .unwrap()
    }
}

pub struct Web3Client;

impl Client for Web3Client {}
