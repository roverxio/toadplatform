use ethers::types::Address;

use crate::contracts::entrypoint_provider::EntryPointProvider;
use crate::errors::ProviderError;
use crate::models::contract_interaction;
use crate::provider::web3_provider::Web3Provider;
use crate::provider::Web3Client;
use crate::CONFIG;

#[derive(Clone)]
pub struct Bundler;

impl Bundler {
    pub async fn submit(
        provider: &Web3Client,
        user_op: contract_interaction::UserOperation,
        beneficiary: Address,
    ) -> Result<String, ProviderError> {
        let call_data = EntryPointProvider::handle_ops(provider, user_op, beneficiary).await?;
        Web3Provider::execute(
            provider.get_bundler_signer(),
            CONFIG.get_chain().entrypoint_address,
            String::from("0"),
            call_data.unwrap(),
            provider.get_entrypoint_provider().abi(),
        )
        .await
        .map_err(|err| ProviderError(err))
    }
}
