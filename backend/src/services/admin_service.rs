use crate::errors::ApiError;
use crate::models::admin::paymaster_topup::PaymasterTopup;
use crate::models::transfer::transfer_response::TransactionResponse;

#[derive(Clone)]
pub struct AdminService {}

impl AdminService {
    pub fn topup_paymaster_deposit(
        &self,
        topup: PaymasterTopup,
    ) -> Result<TransactionResponse, ApiError> {
        println!("topup: {:?}", topup.address);
        Ok(TransactionResponse {
            transaction_hash: "hash".to_string(),
            status: "success".to_string(),
            explorer: "no".to_string(),
        })
    }
}
