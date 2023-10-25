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

#[derive(Clone)]
pub struct Web3Client {
    pub client: Arc<Provider<Http>>,
}

#[mockall::automock]
impl Web3Client {
    pub fn init_client(client: Arc<Provider<Http>>) -> Web3Client {
        Web3Client { client }
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

    pub fn get_bundler_signer(&self) -> SignerMiddleware<Arc<Provider<Http>>, LocalWallet> {
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
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::provider::web3_provider::tests::setup_mock_provider;

    pub fn setup_mock_client() -> Web3Client {
        let provider = setup_mock_provider();
        let mock_client = MockWeb3Client::init_client_context();
        mock_client
            .expect()
            .returning(|client| Web3Client { client });

        MockWeb3Client::init_client(Arc::new(provider.clone()))
    }

    #[tokio::test]
    async fn test_new_client() {
        let provider = setup_mock_provider();

        let mock_client = MockWeb3Client::init_client_context();
        mock_client
            .expect()
            .returning(|client| Web3Client { client });

        let web3_client = MockWeb3Client::init_client(Arc::new(provider.clone()));

        assert_eq!(
            web3_client.client.url(),
            Web3Client {
                client: Arc::new(provider)
            }
            .client
            .url()
        )
    }

    #[tokio::test]
    async fn test_usdc_provider() {
        let provider = setup_mock_provider();

        let mut client = MockWeb3Client::new();
        let abi = USDCProvider::init_abi(Address::zero(), Arc::new(provider.clone()));

        client
            .expect_get_usdc_provider()
            .returning(move || USDCProvider::init_abi(Address::zero(), Arc::new(provider.clone())));

        let usdc_provider = client.get_usdc_provider();

        assert_eq!(usdc_provider.address(), abi.address())
    }
}
