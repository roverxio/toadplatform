use ethers::contract::{Eip712, EthAbiType};
use ethers::types::{Address, Bytes, U256};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, EthAbiType, Eip712, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperation {
    pub sender: Address,
    pub nonce: U256,
    pub init_code: Bytes,
    pub calldata: Bytes,
    pub call_gas_limit: U256,
    pub verification_gas_limit: U256,
    pub pre_verification_gas: U256,
    pub max_fee_per_gas: U256,
    pub max_priority_fee_per_gas: U256,
    pub paymaster_and_data: Bytes,
    pub signature: Bytes
}
