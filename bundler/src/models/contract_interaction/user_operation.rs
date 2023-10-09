use crate::CONFIG;
use ethers::abi::AbiEncode;
use ethers::contract::Eip712;
use ethers::{
    prelude::{EthAbiCodec, EthAbiType},
    types::{Address, Bytes, H256, U256},
    utils::keccak256,
};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// Transaction type for ERC-4337 account abstraction
#[derive(Clone, Debug, Default, Serialize, Deserialize, EthAbiCodec, EthAbiType, Eip712)]
#[serde(rename_all = "camelCase")]
pub struct UserOperation {
    /// Sender of the user operation
    pub sender: Address,

    /// Nonce (anti replay protection)
    pub nonce: U256,

    /// Init code for the account (needed if account not yet deployed and needs to be created)
    pub init_code: Bytes,

    /// The data that is passed to the sender during the main execution call
    pub call_data: Bytes,

    /// The amount of gas to allocate for the main execution call
    pub call_gas_limit: U256,

    /// The amount of gas to allocate for the verification step
    pub verification_gas_limit: U256,

    /// The amount of gas to pay bundler to compensate for the pre-verification execution and calldata
    pub pre_verification_gas: U256,

    /// Maximum fee per gas (similar to EIP-1559)
    pub max_fee_per_gas: U256,

    /// Maximum priority fee per gas (similar to EIP-1559)
    pub max_priority_fee_per_gas: U256,

    /// Address of paymaster sponsoring the user operation, followed by extra data to send to the paymaster (can be empty)
    pub paymaster_and_data: Bytes,

    /// Data passed to the account along with the nonce during the verification step
    pub signature: Bytes,
}

impl UserOperation {
    pub fn new() -> Self {
        Self {
            sender: Default::default(),
            nonce: Default::default(),
            init_code: Default::default(),
            call_data: Default::default(),
            call_gas_limit: U256::from(CONFIG.default_gas.call_gas_limit),
            verification_gas_limit: U256::from(CONFIG.default_gas.verification_gas_limit),
            pre_verification_gas: U256::from(CONFIG.default_gas.pre_verification_gas),
            max_fee_per_gas: U256::from(CONFIG.default_gas.max_fee_per_gas),
            max_priority_fee_per_gas: U256::from(CONFIG.default_gas.max_priority_fee_per_gas),
            paymaster_and_data: Default::default(),
            signature: Default::default(),
        }
    }
    // Builder pattern helpers

    /// Sets the sender of the user operation
    pub fn sender(&mut self, sender: Address) -> &mut UserOperation {
        self.sender = sender;
        self
    }

    /// Sets the nonce of the user operation
    pub fn nonce(&mut self, nonce: u64) -> &mut UserOperation {
        self.nonce = U256::from(nonce);
        self
    }

    /// Sets the init code of the user operation
    pub fn init_code(
        &mut self,
        factory_address: Address,
        create_account_payload: Bytes,
    ) -> &mut UserOperation {
        self.init_code =
            Bytes::from([factory_address.as_bytes(), create_account_payload.as_ref()].concat());
        self
    }

    /// Sets the call data of the user operation
    pub fn call_data(&mut self, call_data: Bytes) -> &mut UserOperation {
        self.call_data = call_data;
        self
    }

    /// Sets the call gas limit of the user operation
    pub fn call_gas_limit(&mut self, call_gas_limit: U256) -> &mut UserOperation {
        self.call_gas_limit = call_gas_limit;
        self
    }

    /// Sets the verification gas limit of the user operation
    pub fn verification_gas_limit(&mut self, verification_gas_limit: U256) -> &mut UserOperation {
        self.verification_gas_limit = verification_gas_limit;
        self
    }

    /// Sets the pre-verification gas of the user operation
    pub fn pre_verification_gas(&mut self, pre_verification_gas: U256) -> &mut UserOperation {
        self.pre_verification_gas = pre_verification_gas;
        self
    }

    /// Sets the paymaster and data of the user operation
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

    /// Sets the signature of the user operation
    pub fn signature(&mut self, signature: Bytes) -> &mut UserOperation {
        self.signature = signature;
        self
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

    pub fn pack_without_signature(&self) -> Bytes {
        let user_operation_packed = UserOperationUnsigned::from(self.clone());
        user_operation_packed.encode().into()
    }
}

/// User operation without signature
#[derive(EthAbiCodec, EthAbiType)]
pub struct UserOperationUnsigned {
    pub sender: Address,
    pub nonce: U256,
    pub init_code: H256,
    pub call_data: H256,
    pub call_gas_limit: U256,
    pub verification_gas_limit: U256,
    pub pre_verification_gas: U256,
    pub max_fee_per_gas: U256,
    pub max_priority_fee_per_gas: U256,
    pub paymaster_and_data: H256,
}

impl From<UserOperation> for UserOperationUnsigned {
    fn from(value: UserOperation) -> Self {
        Self {
            sender: value.sender,
            nonce: value.nonce,
            init_code: keccak256(value.init_code.deref()).into(),
            call_data: keccak256(value.call_data.deref()).into(),
            call_gas_limit: value.call_gas_limit,
            verification_gas_limit: value.verification_gas_limit,
            pre_verification_gas: value.pre_verification_gas,
            max_fee_per_gas: value.max_fee_per_gas,
            max_priority_fee_per_gas: value.max_priority_fee_per_gas,
            paymaster_and_data: keccak256(value.paymaster_and_data.deref()).into(),
        }
    }
}
