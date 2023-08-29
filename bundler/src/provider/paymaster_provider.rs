use ethers::providers::{Http, Provider};
use ethers::utils::format_ether;
use log::error;

use crate::provider::verifying_paymaster_helper::{UserOperation, VerifyingPaymaster};

#[derive(Clone)]
pub struct PaymasterProvider {
    pub provider: VerifyingPaymaster<Provider<Http>>,
}

impl PaymasterProvider {
    pub async fn get_deposit(&self) -> Result<String, String> {
        let response = self.provider.get_deposit().await;
        if response.is_err() {
            error!(
                "Paymaster: Deposit: {:?}",
                response.err().unwrap().to_string()
            );
            return Err(String::from("Failed to get balance"));
        }
        Ok(format_ether(response.unwrap()))
    }

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
