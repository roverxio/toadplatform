use ethers::abi::Abi;
use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::{Address, Bytes, U256};
use std::sync::Arc;

use crate::errors::ProviderError;
use crate::models::contract_interaction;
use crate::provider::Web3Client;

abigen!(EntryPoint, "abi/Entrypoint.json");

#[derive(Clone)]
pub struct EntryPointProvider {
    pub abi: EntryPoint<Provider<Http>>,
}

impl EntryPointProvider {
    pub fn abi(&self) -> &Abi {
        self.abi.abi()
    }

    pub fn init_abi(address: Address, client: Arc<Provider<Http>>) -> EntryPoint<Provider<Http>> {
        let contract: EntryPoint<Provider<Http>> = EntryPoint::new(address, client);
        contract
    }
    pub async fn get_nonce(client: &Web3Client, sender: Address) -> Result<U256, ProviderError> {
        let result = client
            .get_entrypoint_provider()
            .get_nonce(sender, U256::zero())
            .await;
        match result {
            Ok(nonce) => Ok(nonce),
            Err(err) => Err(ProviderError(format!("Failed to get Nonce: {:?}", err))),
        }
    }

    pub async fn add_deposit(
        client: &Web3Client,
        address: Address,
    ) -> Result<Bytes, ProviderError> {
        let data = client
            .get_entrypoint_provider()
            .deposit_to(address)
            .calldata();
        match data {
            Some(call_data) => Ok(call_data),
            None => Err(ProviderError(String::from("EP: Add deposit data failed"))),
        }
    }

    pub async fn handle_ops(
        &self,
        user_op: contract_interaction::user_operation::UserOperation,
        beneficiary: Address,
    ) -> Result<Bytes, String> {
        let data = self
            .abi
            .handle_ops(
                vec![self.get_entry_point_user_operation_payload(user_op)],
                beneficiary,
            )
            .calldata();
        if data.is_none() {
            return Err(String::from("handle ops data failed"));
        }
        Ok(data.unwrap())
    }

    fn get_entry_point_user_operation_payload(
        &self,
        user_op: contract_interaction::user_operation::UserOperation,
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
