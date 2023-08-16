use ethers::abi::AbiEncode;
use ethers::contract::{Eip712, EthAbiType};
use ethers::prelude::EthAbiCodec;
use ethers::types::{Address, Bytes, H256};
use ethers::utils::keccak256;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Clone, Debug, EthAbiType, Eip712, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperation {
    pub sender: Address,
    pub nonce: u64,
    pub init_code: Bytes,
    pub calldata: Bytes,
    pub call_gas_limit: u64,
    pub verification_gas_limit: u64,
    pub pre_verification_gas: u64,
    pub max_fee_per_gas: u64,
    pub max_priority_fee_per_gas: u64,
    pub paymaster_and_data: Bytes,
    pub signature: Bytes,
}

impl UserOperation {
    pub fn pack_without_signature(&self) -> Bytes {
        let user_operation_packed = UserOperationUnsigned::from(self.clone());
        user_operation_packed.encode().into()
    }

    pub fn hash(&self, entry_point: Address, chain_id: u64) -> [u8; 32] {
        keccak256(
            [
                keccak256(self.pack_without_signature().deref()).to_vec(),
                entry_point.encode(),
                chain_id.encode(),
            ]
            .concat(),
        )
    }
}

#[derive(EthAbiCodec, EthAbiType)]
pub struct UserOperationUnsigned {
    pub sender: Address,
    pub nonce: u64,
    pub init_code: H256,
    pub call_data: H256,
    pub call_gas_limit: u64,
    pub verification_gas_limit: u64,
    pub pre_verification_gas: u64,
    pub max_fee_per_gas: u64,
    pub max_priority_fee_per_gas: u64,
    pub paymaster_and_data: H256,
}

impl From<UserOperation> for UserOperationUnsigned {
    fn from(value: UserOperation) -> Self {
        Self {
            sender: value.sender,
            nonce: value.nonce,
            init_code: keccak256(value.init_code.deref()).into(),
            call_data: keccak256(value.calldata.deref()).into(),
            call_gas_limit: value.call_gas_limit,
            verification_gas_limit: value.verification_gas_limit,
            pre_verification_gas: value.pre_verification_gas,
            max_fee_per_gas: value.max_fee_per_gas,
            max_priority_fee_per_gas: value.max_priority_fee_per_gas,
            paymaster_and_data: keccak256(value.paymaster_and_data.deref()).into(),
        }
    }
}
