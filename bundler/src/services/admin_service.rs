use ethers::providers::{Http, Provider};
use ethers::types::Address;
use ethers::utils::parse_ether;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

use crate::constants::Constants;
use crate::contracts::entrypoint_provider::EntryPointProvider;
use crate::contracts::verifying_paymaster_provider::VerifyingPaymasterProvider;
use crate::db::dao::TokenMetadataDao;
use crate::errors::AdminError;
use crate::models::admin::{AddMetadataRequest, MetadataResponse};
use crate::models::transfer::{Status, TransactionResponse, TransferResponse};
use crate::models::wallet::{Balance, BalanceResponse};
use crate::models::Metadata;
use crate::provider::web3_provider::Web3Provider;
use crate::provider::*;
use crate::CONFIG;

#[derive(Clone)]
pub struct AdminService;

impl AdminService {
    pub async fn topup_paymaster_deposit(
        provider: &Arc<Provider<Http>>,
        eth_value: String,
        paymaster: String,
        metadata: Metadata,
    ) -> Result<TransferResponse, AdminError> {
        if metadata.currency != Constants::NATIVE {
            return Err(AdminError::InvalidCurrency);
        }
        if paymaster != Constants::VERIFYING_PAYMASTER {
            return Err(AdminError::ValidationError(String::from(
                "Invalid Paymaster",
            )));
        }
        let value = parse_ether(eth_value)
            .map_err(|_| AdminError::ValidationError(String::from("Invalid value")))?;

        let data = EntryPointProvider::add_deposit(
            provider,
            CONFIG.get_chain().verifying_paymaster_address,
        )
        .await?;
        let response = Web3Provider::execute(
            Web3Client::get_relayer_signer(provider.clone()),
            CONFIG.get_chain().entrypoint_address,
            value.to_string(),
            data,
            Web3Client::get_entrypoint_provider(provider.clone()).abi(),
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
            Err(err) => Err(AdminError::Provider(err)),
        }
    }

    pub async fn get_balance(
        provider: &Arc<Provider<Http>>,
        entity: String,
        data: Balance,
    ) -> Result<BalanceResponse, AdminError> {
        if data.currency != Constants::NATIVE {
            return Err(AdminError::InvalidCurrency);
        }
        if Constants::PAYMASTER == entity {
            let paymaster_address = &CONFIG.get_chain().verifying_paymaster_address;
            let deposit = VerifyingPaymasterProvider::get_deposit(provider).await?;
            return Self::get_balance_response(paymaster_address, deposit, data.currency);
        }
        if Constants::RELAYER == entity {
            let relayer_address = &CONFIG.run_config.account_owner;
            let balance = Web3Provider::get_balance(relayer_address.clone()).await?;
            return Self::get_balance_response(relayer_address, balance, data.currency);
        }
        Err(AdminError::ValidationError(String::from("Invalid entity")))
    }

    pub async fn add_currency_metadata(
        pool: &Pool<Postgres>,
        metadata: AddMetadataRequest,
    ) -> Result<MetadataResponse, AdminError> {
        TokenMetadataDao::add_metadata(
            pool,
            metadata.get_chain_name().clone(),
            metadata.get_symbol(),
            metadata.get_contract_address(),
            metadata.get_exponent(),
            metadata.get_token_type(),
            metadata.get_token_name(),
            metadata.get_chain_id(),
            metadata.get_chain_display_name(),
            metadata.get_token_image_url(),
        )
        .await?;

        let supported_currencies =
            TokenMetadataDao::get_metadata_for_chain(pool, metadata.get_chain_name(), None).await?;

        let exponent_metadata = MetadataResponse::new().to(
            supported_currencies.clone(),
            supported_currencies[0].chain.clone(),
            CONFIG.chains[&supported_currencies[0].chain.clone()].chain_id,
            CONFIG.chains[&supported_currencies[0].chain.clone()]
                .currency
                .clone(),
        );

        Ok(exponent_metadata)
    }

    fn get_balance_response(
        address: &Address,
        balance: String,
        currency: String,
    ) -> Result<BalanceResponse, AdminError> {
        Ok(BalanceResponse::new(
            balance,
            format!("{:?}", address),
            currency,
            0, // sending parsed eth here for ease of readability
        ))
    }
}
