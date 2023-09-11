use crate::CONFIG;
use ethers::abi::AbiEncode;
use ethers::contract::{Eip712, EthAbiType};
use ethers::prelude::EthAbiCodec;
use ethers::types::{Address, Bytes, H256};
use ethers::utils::keccak256;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Clone, Default, Debug, EthAbiType, Eip712, Serialize, Deserialize)]
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
    pub fn new() -> Self {
        Self {
            sender: Default::default(),
            nonce: Default::default(),
            init_code: Default::default(),
            calldata: Default::default(),
            call_gas_limit: CONFIG.default_gas.call_gas_limit,
            verification_gas_limit: CONFIG.default_gas.verification_gas_limit,
            pre_verification_gas: CONFIG.default_gas.pre_verification_gas,
            max_fee_per_gas: CONFIG.default_gas.max_fee_per_gas,
            max_priority_fee_per_gas: CONFIG.default_gas.max_priority_fee_per_gas,
            paymaster_and_data: Default::default(),
            signature: Default::default(),
        }
    }

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

    pub fn init_code(
        &mut self,
        factory_address: Address,
        create_account_payload: Bytes,
    ) -> &mut UserOperation {
        self.init_code =
            Bytes::from([factory_address.as_bytes(), create_account_payload.as_ref()].concat());
        self
    }

    pub fn calldata(&mut self, calldata: Bytes) -> &mut UserOperation {
        self.calldata = calldata;
        self
    }

    pub fn nonce(&mut self, nonce: u64) -> &mut UserOperation {
        self.nonce = nonce;
        self
    }

    pub fn sender(&mut self, wallet_address: Address) -> &mut UserOperation {
        self.sender = wallet_address;
        self
    }

    pub fn paymaster_and_data(
        &mut self,
        data: ethers::abi::Bytes,
        paymaster: Address,
        sign: Option<Vec<u8>>,
    ) -> &mut UserOperation {
        self.paymaster_and_data =
            Bytes::from([paymaster.as_bytes(), &data, &sign.unwrap_or(vec![0u8; 65])].concat());
        self
    }

    pub fn signature(&mut self, signature: Bytes) -> &mut UserOperation {
        self.signature = signature;
        self
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
