use log::info;

use crate::errors::ApiError;
use crate::models::admin::paymaster_topup::PaymasterTopup;
use crate::models::transfer::transfer_response::TransactionResponse;
use crate::models::wallet::balance_response::BalanceResponse;

#[derive(Clone)]
pub struct AdminService {}

impl AdminService {
    pub fn topup_paymaster_deposit(
        &self,
        topup: PaymasterTopup,
    ) -> Result<TransactionResponse, ApiError> {
        info!("topup: {:?}", topup.address);
        Ok(TransactionResponse {
            transaction_hash: "hash".to_string(),
            status: "success".to_string(),
            explorer: "no".to_string(),
        })
    }

    pub fn get_balance(&self, entity: String) -> Result<BalanceResponse, ApiError> {
        info!("entity: {:?}", entity);
        Ok(BalanceResponse {
            balance: "".to_string(),
            address: "".to_string(),
            currency: "".to_string(),
        })
    }
}
