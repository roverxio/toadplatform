use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Provider};
use ethers::types::Address;
use ethers_signers::LocalWallet;
use std::sync::Arc;

use crate::contracts::entrypoint_provider::EntryPointProvider;
use crate::models::contract_interaction;
use crate::provider::web3_provider::Web3Provider;
use crate::CONFIG;

#[derive(Clone)]
pub struct Bundler {
    pub signer: SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
    pub entrypoint: EntryPointProvider,
}

impl Bundler {
    pub async fn submit(
        signer: SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
        entrypoint: EntryPointProvider,
        user_op: contract_interaction::user_operation::UserOperation,
        beneficiary: Address,
    ) -> Result<String, String> {
        let call_data = entrypoint.handle_ops(user_op, beneficiary).await;
        if call_data.is_err() {
            return Err(String::from("failed to transfer"));
        }
        Web3Provider::execute(
            signer.clone(),
            CONFIG.get_chain().entrypoint_address,
            String::from("0"),
            call_data.unwrap(),
            entrypoint.abi(),
        )
        .await
    }
}
