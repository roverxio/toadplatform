use ethers::prelude::{Http, Provider};
use ethers::types::Address;
use std::sync::Arc;

use crate::contracts::entrypoint_provider::EntryPointProvider;
use crate::errors::ProviderError;
use crate::models::contract_interaction;
use crate::provider::web3_provider::Web3Provider;
use crate::provider::*;
use crate::CONFIG;

#[derive(Clone)]
pub struct Bundler;

impl Bundler {
    pub async fn submit(
        provider: &Arc<Provider<Http>>,
        user_op: contract_interaction::UserOperation,
        beneficiary: Address,
    ) -> Result<String, ProviderError> {
        let call_data =
            EntryPointProvider::handle_ops(&provider.clone(), user_op, beneficiary).await?;
        Web3Provider::execute(
            Web3Client::get_bundler_signer(provider.clone()),
            CONFIG.get_chain().entrypoint_address,
            String::from("0"),
            call_data,
            Web3Client::get_entrypoint_provider(provider.clone()).abi(),
        )
        .await
        .map_err(|err| ProviderError(err))
    }
}
