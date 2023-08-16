use crate::CONFIG;
use ethers::abi::Abi;
use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::{Address, U256};
use std::sync::Arc;

use crate::models::contract_interaction;
use crate::provider::web3_provider::Web3Provider;

abigen!(EntryPoint, "abi/Entrypoint.json");

#[derive(Clone)]
pub struct EntryPointProvider {
    pub provider: EntryPoint<Provider<Http>>,
    pub client: Web3Provider,
}

pub fn get_entrypoint_abi(
    current_chain: &str,
    client: Arc<Provider<Http>>,
) -> EntryPoint<Provider<Http>> {
    let contract: EntryPoint<Provider<Http>> =
        EntryPoint::new(CONFIG.chains[current_chain].entrypoint_address, client);
    contract
}

// UserOperation is local to EntryPoint
pub fn get_entry_point_user_operation_payload(
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

impl EntryPointProvider {
    pub fn get_address(&self) -> Address {
        self.provider.address()
    }

    pub async fn get_nonce(&self, sender: Address) -> Result<U256, String> {
        let result = self.provider.get_nonce(sender, U256::zero()).await;
        if result.is_err() {
            return Err(String::from("failed to get Nonce"));
        }
        Ok(result.unwrap())
    }

    pub async fn add_deposit(&self, address: Address, value: String) -> Result<String, String> {
        let data = self.provider.deposit_to(address).calldata();
        if data.is_none() {
            return Err(String::from("failed to deposit"));
        }
        let abi: &Abi = self.provider.abi();
        let result = self
            .client
            .execute(
                CONFIG.run_config.account_owner,
                CONFIG.chains[&CONFIG.run_config.current_chain].entrypoint_address,
                value,
                data.unwrap(),
                abi,
            )
            .await;
        match result {
            Ok(tx_hash) => Ok(tx_hash),
            Err(error) => Err(error),
        }
    }

    pub async fn handle_ops(
        &self,
        user_op: contract_interaction::user_operation::UserOperation,
    ) -> Result<String, String> {
        let data = self
            .provider
            .handle_ops(
                vec![get_entry_point_user_operation_payload(user_op)],
                CONFIG.run_config.account_owner,
            )
            .calldata();
        if data.is_none() {
            return Err(String::from("failed to execute"));
        }
        let result = self
            .client
            .execute(
                CONFIG.run_config.account_owner,
                CONFIG.chains[&CONFIG.run_config.current_chain].entrypoint_address,
                String::from("0"),
                data.unwrap(),
                self.provider.abi(),
            )
            .await;
        match result {
            Ok(tx_hash) => Ok(tx_hash),
            Err(error) => Err(error),
        }
    }
}
