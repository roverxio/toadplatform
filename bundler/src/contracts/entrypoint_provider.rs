use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::{Address, Bytes, U256};
use std::sync::Arc;

use crate::errors::ProviderError;
use crate::models::contract_interaction;
use crate::provider::*;

abigen!(EntryPoint, "abi/Entrypoint.json");

#[derive(Clone)]
pub struct EntryPointProvider;

impl EntryPointProvider {
    pub fn init_abi(address: Address, client: Arc<Provider<Http>>) -> EntryPoint<Provider<Http>> {
        let contract: EntryPoint<Provider<Http>> = EntryPoint::new(address, client);
        contract
    }

    pub async fn get_nonce(
        client: &Arc<Provider<Http>>,
        sender: Address,
    ) -> Result<U256, ProviderError> {
        let result = Web3Client::get_entrypoint_provider(client.clone())
            .get_nonce(sender, U256::zero())
            .await;
        match result {
            Ok(nonce) => Ok(nonce),
            Err(err) => Err(ProviderError(format!("Failed to get Nonce: {:?}", err))),
        }
    }

    pub async fn add_deposit(
        client: &Arc<Provider<Http>>,
        address: Address,
    ) -> Result<Bytes, ProviderError> {
        let data = Web3Client::get_entrypoint_provider(client.clone())
            .deposit_to(address)
            .calldata();
        match data {
            Some(call_data) => Ok(call_data),
            None => Err(ProviderError(String::from("EP: Add deposit data failed"))),
        }
    }

    pub async fn handle_ops(
        client: &Arc<Provider<Http>>,
        user_op: contract_interaction::UserOperation,
        beneficiary: Address,
    ) -> Result<Bytes, ProviderError> {
        let data = Web3Client::get_entrypoint_provider(client.clone())
            .handle_ops(
                vec![Self::get_entry_point_user_operation_payload(user_op)],
                beneficiary,
            )
            .calldata();
        match data {
            Some(call_data) => Ok(call_data),
            None => Err(ProviderError(String::from("handle ops data failed"))),
        }
    }

    fn get_entry_point_user_operation_payload(
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
