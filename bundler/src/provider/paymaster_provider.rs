use ethers::providers::{Http, Provider};
use log::error;

use crate::contracts::verifying_paymaster_provider::{UserOperation, VerifyingPaymaster};

#[derive(Clone)]
pub struct PaymasterProvider {
    pub provider: VerifyingPaymaster<Provider<Http>>,
}

impl PaymasterProvider {
    pub async fn get_hash(
        &self,
        user_operation: UserOperation,
        valid_until: u64,
        valid_after: u64,
    ) -> Result<[u8; 32], String> {
        let response = self
            .provider
            .get_hash(user_operation, valid_until, valid_after)
            .await;
        if response.is_err() {
            error!("Paymaster: Hash: {:?}", response.err().unwrap().to_string());
            return Err(String::from("Failed to get hash"));
        }
        Ok(response.unwrap())
    }
}
