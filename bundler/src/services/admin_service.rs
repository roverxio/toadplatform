use ethers::addressbook::Address;
use log::info;

use crate::CONFIG;
use crate::constants::Constants;
use crate::errors::ApiError;
use crate::models::admin::paymaster_topup::PaymasterTopup;
use crate::models::transfer::transfer_response::TransactionResponse;
use crate::models::wallet::balance_request::Balance;
use crate::models::wallet::balance_response::BalanceResponse;
use crate::provider::paymaster_provider::PaymasterProvider;
use crate::provider::web3_provider::Web3Provider;

#[derive(Clone)]
pub struct AdminService {
    pub paymaster_provider: PaymasterProvider,
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
        if data.currency != Constants::NATIVE {
            return Err(ApiError::BadRequest("Invalid currency".to_string()));
        }
        if Constants::PAYMASTER == entity {
            let paymaster_address = &CONFIG.chains[&CONFIG.run_config.current_chain].verifying_paymaster_address;
            let response = self.paymaster_provider.get_deposit().await;
            return Self::get_balance_response(paymaster_address, response, data.currency);
        }
        if RoverXConstants::RELAYER == entity {
            let relayer_address = &CONFIG.run_config.account_owner;
            let response = Web3Provider::get_native_balance(relayer_address.clone()).await;
            return Self::get_balance_response(relayer_address, response, data.currency);
        }
        Err(ApiError::BadRequest("Invalid entity".to_string()))
    }

    fn get_balance_response(address: &Address, response: Result<String, String>, currency: String) -> Result<BalanceResponse, ApiError> {
        return match response {
            Ok(balance) => {
                Ok(BalanceResponse::new(balance, format!("{:?}", address), currency))
            }
            Err(error) => Err(ApiError::InternalServer(error))
        };
    }
}
