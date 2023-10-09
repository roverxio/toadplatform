use ethers::{
    prelude::{EthAbiCodec, EthAbiType},
    types::{Address, Bytes, Log, TransactionReceipt, H256, U256, U64},
    utils::keccak256,
};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// Transaction type for ERC-4337 account abstraction
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    EthAbiCodec,
    EthAbiType,
)]
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
    // Builder pattern helpers

    /// Sets the sender of the user operation
    pub fn sender(mut self, sender: Address) -> Self {
        self.sender = sender;
        self
    }

    /// Sets the nonce of the user operation
    pub fn nonce(mut self, nonce: U256) -> Self {
        self.nonce = nonce;
        self
    }

    /// Sets the init code of the user operation
    pub fn init_code(mut self, init_code: Bytes) -> Self {
        self.init_code = init_code;
        self
    }

    /// Sets the call data of the user operation
    pub fn call_data(mut self, call_data: Bytes) -> Self {
        self.call_data = call_data;
        self
    }

    /// Sets the call gas limit of the user operation
    pub fn call_gas_limit(mut self, call_gas_limit: U256) -> Self {
        self.call_gas_limit = call_gas_limit;
        self
    }

    /// Sets the verification gas limit of the user operation
    pub fn verification_gas_limit(mut self, verification_gas_limit: U256) -> Self {
        self.verification_gas_limit = verification_gas_limit;
        self
    }

    /// Sets the pre-verification gas of the user operation
    pub fn pre_verification_gas(mut self, pre_verification_gas: U256) -> Self {
        self.pre_verification_gas = pre_verification_gas;
        self
    }

    /// Sets the max fee per gas of the user operation
    pub fn max_fee_per_gas(mut self, max_fee_per_gas: U256) -> Self {
        self.max_fee_per_gas = max_fee_per_gas;
        self
    }

    /// Sets the max priority fee per gas of the user operation
    pub fn max_priority_fee_per_gas(mut self, max_priority_fee_per_gas: U256) -> Self {
        self.max_priority_fee_per_gas = max_priority_fee_per_gas;
        self
    }

    /// Sets the paymaster and data of the user operation
    pub fn paymaster_and_data(mut self, paymaster_and_data: Bytes) -> Self {
        self.paymaster_and_data = paymaster_and_data;
        self
    }

    /// Sets the signature of the user operation
    pub fn signature(mut self, signature: Bytes) -> Self {
        self.signature = signature;
        self
    }

    /// Creates random user operation (for testing purposes)
    #[cfg(feature = "test-utils")]
    pub fn random() -> Self {
        UserOperation::default()
            .sender(Address::random())
            .verification_gas_limit(100_000.into())
            .pre_verification_gas(21_000.into())
            .max_priority_fee_per_gas(1_000_000_000.into())
    }
}

/// User operation hash
#[derive(
    Eq, Hash, PartialEq, Debug, Serialize, Deserialize, Clone, Copy, Default, PartialOrd, Ord,
)]
pub struct UserOperationHash(pub H256);

impl From<H256> for UserOperationHash {
    fn from(value: H256) -> Self {
        Self(value)
    }
}

impl From<UserOperationHash> for H256 {
    fn from(value: UserOperationHash) -> Self {
        value.0
    }
}

impl From<[u8; 32]> for UserOperationHash {
    fn from(value: [u8; 32]) -> Self {
        Self(H256::from_slice(&value))
    }
}

impl UserOperationHash {
    #[inline]
    pub const fn as_fixed_bytes(&self) -> &[u8; 32] {
        &self.0 .0
    }

    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0 .0
    }

    #[inline]
    pub const fn repeat_byte(byte: u8) -> UserOperationHash {
        UserOperationHash(H256([byte; 32]))
    }

    #[inline]
    pub const fn zero() -> UserOperationHash {
        UserOperationHash::repeat_byte(0u8)
    }

    pub fn assign_from_slice(&mut self, src: &[u8]) {
        self.as_bytes_mut().copy_from_slice(src);
    }

    pub fn from_slice(src: &[u8]) -> Self {
        let mut ret = Self::zero();
        ret.assign_from_slice(src);
        ret
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

/// Receipt of the user operation (returned from the RPC endpoint eth_getUserOperationReceipt)
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperationReceipt {
    #[serde(rename = "userOpHash")]
    pub user_operation_hash: UserOperationHash,
    pub sender: Address,
    pub nonce: U256,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paymaster: Option<Address>,
    pub actual_gas_cost: U256,
    pub actual_gas_used: U256,
    pub success: bool,
    pub reason: String,
    pub logs: Vec<Log>,
    #[serde(rename = "receipt")]
    pub tx_receipt: TransactionReceipt,
}

/// Struct that is returned from the RPC endpoint eth_getUserOperationByHash
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperationByHash {
    pub user_operation: UserOperation,
    pub entry_point: Address,
    pub transaction_hash: H256,
    pub block_hash: H256,
    pub block_number: U64,
}

/// User operation with all fields being optional
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperationPartial {
    pub sender: Option<Address>,
    pub nonce: Option<U256>,
    pub init_code: Option<Bytes>,
    pub call_data: Option<Bytes>,
    pub call_gas_limit: Option<U256>,
    pub verification_gas_limit: Option<U256>,
    pub pre_verification_gas: Option<U256>,
    pub max_fee_per_gas: Option<U256>,
    pub max_priority_fee_per_gas: Option<U256>,
    pub paymaster_and_data: Option<Bytes>,
    pub signature: Option<Bytes>,
}

impl From<UserOperationPartial> for UserOperation {
    fn from(user_operation: UserOperationPartial) -> Self {
        Self {
            sender: {
                if let Some(sender) = user_operation.sender {
                    sender
                } else {
                    Address::zero()
                }
            },
            nonce: {
                if let Some(nonce) = user_operation.nonce {
                    nonce
                } else {
                    U256::zero()
                }
            },
            init_code: {
                if let Some(init_code) = user_operation.init_code {
                    init_code
                } else {
                    Bytes::default()
                }
            },
            call_data: {
                if let Some(call_data) = user_operation.call_data {
                    call_data
                } else {
                    Bytes::default()
                }
            },
            call_gas_limit: {
                if let Some(call_gas_limit) = user_operation.call_gas_limit {
                    call_gas_limit
                } else {
                    U256::zero()
                }
            },
            verification_gas_limit: {
                if let Some(verification_gas_limit) = user_operation.verification_gas_limit {
                    verification_gas_limit
                } else {
                    U256::zero()
                }
            },
            pre_verification_gas: {
                if let Some(pre_verification_gas) = user_operation.pre_verification_gas {
                    pre_verification_gas
                } else {
                    U256::zero()
                }
            },
            max_fee_per_gas: {
                if let Some(max_fee_per_gas) = user_operation.max_fee_per_gas {
                    max_fee_per_gas
                } else {
                    U256::zero()
                }
            },
            max_priority_fee_per_gas: {
                if let Some(max_priority_fee_per_gas) = user_operation.max_priority_fee_per_gas {
                    max_priority_fee_per_gas
                } else {
                    U256::zero()
                }
            },
            paymaster_and_data: {
                if let Some(paymaster_and_data) = user_operation.paymaster_and_data {
                    paymaster_and_data
                } else {
                    Bytes::default()
                }
            },
            signature: {
                if let Some(signature) = user_operation.signature {
                    signature
                } else {
                    Bytes::default()
                }
            },
        }
    }
}
