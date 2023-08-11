use log::info;

use ethers::providers::{Http, Provider};

use crate::CONFIG;
use crate::constants::RoverXConstants;
use crate::errors::ApiError;
use crate::models::admin::paymaster_topup::PaymasterTopup;
use crate::models::transfer::transfer_response::TransactionResponse;
use crate::models::wallet::balance_request::Balance;
use crate::models::wallet::balance_response::BalanceResponse;
use crate::provider::entrypoint_helper::EntryPoint;

#[derive(Clone)]
pub struct AdminService {
    pub entrypoint_provider: EntryPoint<Provider<Http>>,
}

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

    pub async fn get_balance(&self, entity: String, data: Balance) -> Result<BalanceResponse, ApiError> {
        if data.currency != RoverXConstants::NATIVE {
            return Err(ApiError::BadRequest("Invalid currency".to_string()));
        }
        if RoverXConstants::PAYMASTER == entity {
            let paymaster_address = &CONFIG.chains[&CONFIG.current_chain].verifying_paymaster_address;
            let deposit = self.entrypoint_provider.get_deposit_info(paymaster_address.clone()).await.unwrap();
            let balance = (deposit.deposit.to_string().parse::<f64>().unwrap() / 1e18).to_string();
            return Ok(BalanceResponse {
                balance,
                address: format!("{:?}", paymaster_address),
                currency: data.currency,
            });
        }
        Err(ApiError::BadRequest("Invalid entity".to_string()))
    }
}
