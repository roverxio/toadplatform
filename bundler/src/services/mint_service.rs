use ethers::types::Address;
use log::{error, info};

use crate::contracts::usdc_provider::USDCProvider;
use crate::models::config::env::ENV;
use crate::provider::web3_provider::Web3Provider;
use crate::provider::Web3Client;
use crate::CONFIG;

#[derive(Clone)]
pub struct MintService;

impl MintService {
    pub async fn mint(provider: Web3Client, receiver: Address) {
        match CONFIG.env {
            ENV::Production => {
                error!("minting is disabled in production");
                return;
            }
            _ => {}
        }
        info!("minting for {:?}", receiver);
        let call_data =
            USDCProvider::mint(&provider.clone(), receiver, "100000000".to_string()).unwrap();
        let response = Web3Provider::execute(
            provider.get_relayer_signer(),
            CONFIG.get_chain().usdc_address,
            "0".to_string(),
            call_data,
            provider.get_usdc_provider().abi(),
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
}
