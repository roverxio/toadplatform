use crate::models::contract_interaction;
use crate::provider::entrypoint_helper::{EntryPoint, UserOperation};
use crate::CONFIG;
use ethers::abi::Abi;
use ethers::providers::{Http, Provider};
use ethers::types::{Address, Bytes, U256};

#[derive(Clone)]
pub struct EntryPointProvider {
    pub abi: EntryPoint<Provider<Http>>,
}

impl EntryPointProvider {
    pub fn abi(&self) -> &Abi {
        self.abi.abi()
    }

    pub async fn get_nonce(&self, sender: Address) -> Result<U256, String> {
        let result = self.abi.get_nonce(sender, U256::zero()).await;
        if result.is_err() {
            return Err(String::from("failed to get Nonce"));
        }
        Ok(result.unwrap())
    }

    pub async fn add_deposit(&self, address: Address) -> Result<Bytes, String> {
        let data = self.abi.deposit_to(address).calldata();
        if data.is_none() {
            return Err(String::from("add deposit data failed"));
        }
        Ok(data.unwrap())
    }

    pub async fn handle_ops(
        &self,
        user_op: contract_interaction::user_operation::UserOperation,
    ) -> Result<Bytes, String> {
        let data = self
            .abi
            .handle_ops(
                vec![self.get_entry_point_user_operation_payload(user_op)],
                CONFIG.run_config.account_owner,
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
