use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::Address;
use ethers::types::U256;
use ethers::utils::format_ether;
use std::sync::Arc;

use crate::errors::ProviderError;
use crate::models::contract_interaction;
use crate::provider::Web3Client;

abigen!(VerifyingPaymaster, "abi/VerifyingPaymaster.json");

pub struct VerifyingPaymasterProvider;

impl VerifyingPaymasterProvider {
    pub fn init_abi(
        address: Address,
        client: Arc<Provider<Http>>,
    ) -> VerifyingPaymaster<Provider<Http>> {
        let contract: VerifyingPaymaster<Provider<Http>> = VerifyingPaymaster::new(address, client);
        contract
    }

    pub async fn get_deposit(client: &Web3Client) -> Result<String, ProviderError> {
        let response = client
            .get_verifying_paymaster_provider()
            .get_deposit()
            .await;
        match response {
            Ok(deposit) => Ok(format_ether(deposit)),
            Err(err) => Err(ProviderError(format!(
                "Paymaster: Deposit: {:?}",
                err.to_string()
            ))),
        }
    }

    pub async fn get_hash(
        client: &Web3Client,
        user_operation: UserOperation,
        valid_until: u64,
        valid_after: u64,
    ) -> Result<[u8; 32], ProviderError> {
        let response = client
            .get_verifying_paymaster_provider()
            .get_hash(user_operation, valid_until, valid_after)
            .await;
        match response {
            Ok(hash) => Ok(hash),
            Err(err) => Err(ProviderError(format!("Paymaster: Hash: {:?}", err))),
        }
    }

    pub fn get_verifying_paymaster_user_operation_payload(
        user_op: contract_interaction::UserOperation,
    ) -> UserOperation {
        UserOperation {
            sender: user_op.sender,
            nonce: U256::from(user_op.nonce),
            init_code: user_op.init_code,
            call_data: user_op.calldata,
            call_gas_limit: U256::from(user_op.call_gas_limit),
            verification_gas_limit: U256::from(user_op.verification_gas_limit),
            pre_verification_gas: U256::from(user_op.pre_verification_gas),
            max_fee_per_gas: U256::from(user_op.max_fee_per_gas),
            max_priority_fee_per_gas: U256::from(user_op.max_priority_fee_per_gas),
            signature: user_op.signature,
            paymaster_and_data: user_op.paymaster_and_data,
        }
    }
}
