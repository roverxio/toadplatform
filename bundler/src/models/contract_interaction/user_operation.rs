use ethers::contract::{Eip712, EthAbiType};
use ethers::types::{Address, Bytes};
use serde::{Deserialize, Serialize};

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
