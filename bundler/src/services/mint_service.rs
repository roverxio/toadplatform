use crate::contracts::usdc_provider::USDCProvider;
use crate::models::config::env::ENV;
use crate::provider::web3_provider::Web3Provider;
use crate::CONFIG;
use ethers::prelude::SignerMiddleware;
use ethers::providers::{Http, Provider};
use ethers::types::Address;
use ethers_signers::LocalWallet;
use log::{error, info};
use std::sync::Arc;

#[derive(Clone)]
pub struct MintService {
    pub usdc_provider: USDCProvider,
    pub signer: SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
}

pub async fn mint(
    receiver: Address,
    usdc_provider: USDCProvider,
    signer: SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
) {
    match CONFIG.env {
        ENV::Production => {
            error!("minting is disabled in production");
            return;
        }
        _ => {}
    }
    info!("minting for {:?}", receiver);
    let calldata = usdc_provider
        .mint(receiver, "100000000".to_string())
        .unwrap();
    let response = Web3Provider::execute(
        signer.clone(),
        CONFIG.get_chain().usdc_address,
        "0".to_string(),
        calldata,
        usdc_provider.abi(),
    )
    .await;
    match response {
        Ok(txn_hash) => {
            info!(
                "wallet {:?} was sent 100000000 USDC {:?} txn_hash -> {:?}",
                receiver,
                CONFIG.get_chain().usdc_address.clone(),
                txn_hash
            );
        }
        Err(err) => {
            error!("mint failed for {:?}. Error: {}", receiver, err)
        }
    }
}
