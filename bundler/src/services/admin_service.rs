use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Provider};
use ethers::types::Address;
use ethers_signers::LocalWallet;
use std::sync::Arc;

use crate::constants::Constants;
use crate::contracts::entrypoint_provider::EntryPointProvider;
use crate::errors::ApiError;
use crate::models::metadata::Metadata;
use crate::models::transfer::status::Status;
use crate::models::transfer::transaction_response::TransactionResponse;
use crate::models::transfer::transfer_response::TransferResponse;
use crate::models::wallet::balance_request::Balance;
use crate::models::wallet::balance_response::BalanceResponse;
use crate::provider::paymaster_provider::PaymasterProvider;
use crate::provider::web3_provider::Web3Provider;
use crate::CONFIG;

#[derive(Clone)]
pub struct AdminService {
    pub paymaster_provider: PaymasterProvider,
    pub entrypoint_provider: EntryPointProvider,
    pub relayer_signer: SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
}

impl AdminService {
    pub async fn topup_paymaster_deposit(
        &self,
        value: String,
        paymaster: String,
        metadata: Metadata,
    ) -> Result<TransferResponse, ApiError> {
        if metadata.currency != Constants::NATIVE {
            return Err(ApiError::BadRequest("Invalid currency".to_string()));
        }
        if paymaster != Constants::VERIFYING_PAYMASTER {
            return Err(ApiError::BadRequest("Invalid Paymaster".to_string()));
        }

        let data = self
            .entrypoint_provider
            .add_deposit(CONFIG.get_chain().verifying_paymaster_address)
            .await;
        if data.is_err() {
            return Err(ApiError::BadRequest(String::from("failed to topup")));
        }
        let response = Web3Provider::execute(
            self.relayer_signer.clone(),
            CONFIG.get_chain().entrypoint_address,
            value,
            data.unwrap(),
            self.entrypoint_provider.abi(),
        )
        .await;
        match response {
            Ok(txn_hash) => Ok(TransferResponse {
                transaction: TransactionResponse::new(
                    txn_hash.clone(),
                    Status::PENDING,
                    CONFIG.get_chain().explorer_url.clone() + &txn_hash.clone(),
                ),
                transaction_id: "".to_string(),
            }),
            Err(err) => Err(ApiError::BadRequest(err)),
        }
    }

    pub async fn get_balance(
        &self,
        entity: String,
        data: Balance,
    ) -> Result<BalanceResponse, ApiError> {
        if data.currency != Constants::NATIVE {
            return Err(ApiError::BadRequest("Invalid currency".to_string()));
        }
        if Constants::PAYMASTER == entity {
            let paymaster_address = &CONFIG.get_chain().verifying_paymaster_address;
            let response = self.paymaster_provider.get_deposit().await;
            return Self::get_balance_response(paymaster_address, response, data.currency);
        }
        if Constants::RELAYER == entity {
            let relayer_address = &CONFIG.run_config.account_owner;
            let response = Web3Provider::get_balance(relayer_address.clone()).await;
            return Self::get_balance_response(relayer_address, response, data.currency);
        }
        Err(ApiError::BadRequest("Invalid entity".to_string()))
    }

    fn get_balance_response(
        address: &Address,
        response: Result<String, String>,
        currency: String,
    ) -> Result<BalanceResponse, ApiError> {
        return match response {
            Ok(balance) => Ok(BalanceResponse::new(
                balance,
                format!("{:?}", address),
                currency,
            )),
            Err(error) => Err(ApiError::InternalServer(error)),
        };
    }
}
