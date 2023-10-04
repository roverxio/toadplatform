use ethers::abi::Abi;
use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::{Address, Bytes, U256};
use std::sync::Arc;

use crate::models::contract_interaction;
use crate::provider::web3_client::Web3Client;

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
    pub async fn get_nonce(&self, sender: Address) -> Result<U256, String> {
        let result = self.abi.get_nonce(sender, U256::zero()).await;
        if result.is_err() {
            return Err(String::from("failed to get Nonce"));
        }
        Ok(result.unwrap())
    }

    pub async fn add_deposit(client: &Web3Client, address: Address) -> Result<Bytes, String> {
        let data = client
            .get_entrypoint_provider()
            .deposit_to(address)
            .calldata();
        if data.is_none() {
            return Err(String::from("add deposit data failed"));
        }
        Ok(data.unwrap())
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
