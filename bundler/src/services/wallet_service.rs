use actix_web::rt::spawn;
use bigdecimal::{BigDecimal, Zero};
use ethers::types::Address;
use log::info;
use sqlx::{Pool, Postgres};
use std::time::SystemTime;

use crate::contracts::simple_account_factory_provider::SimpleAccountFactoryProvider;
use crate::contracts::simple_account_provider::SimpleAccountProvider;
use crate::db::dao::transaction_dao::TransactionDao;
use crate::db::dao::wallet_dao::{User, WalletDao};
use crate::errors::base::ProviderError;
use crate::errors::errors::ApiError;
use crate::models::transaction::transaction::Transaction;
use crate::models::wallet::address_response::AddressResponse;
use crate::provider::helpers::{contract_exists_at, get_hash};
use crate::provider::web3_client::Web3Client;
use crate::services::mint_service::MintService;
use crate::CONFIG;

#[derive(Clone)]
pub struct WalletService {
    pub transaction_dao: TransactionDao,
}

impl WalletService {
    pub async fn get_wallet_address(
        pool: &Pool<Postgres>,
        provider: &Web3Client,
        user: User,
        user_wallet: String,
    ) -> Result<AddressResponse, ApiError> {
        let result: Wallet;
        if user.wallet_address.is_empty() {
            result = Self::get_address(
                &provider.clone(),
                user.external_user_id.as_str(),
                user_wallet.parse().unwrap(),
            )
            .await
            .unwrap();
            info!("salt -> {}", result.salt);
            WalletDao::create_wallet(
                pool,
                user.email,
                user.name,
                format!("{:?}", result.address),
                user_wallet,
                user.external_user_id,
                result.salt,
                result.deployed,
            )
            .await
            .map_err(|_| ApiError::InternalServer("Failed to create wallet".to_string()))?;
            // spawn a thread to mint for user
            spawn(MintService::mint(provider.clone(), result.address.clone()));
        } else {
            result = Wallet {
                address: user.wallet_address.parse().unwrap(),
                salt: BigDecimal::zero(),
                deployed: user.deployed,
            }
        }

        Ok(AddressResponse {
            address: result.address,
        })
    }

    async fn get_address(
        provider: &Web3Client,
        external_user_id: &str,
        user_wallet: Address,
    ) -> Result<Wallet, ProviderError> {
        let mut contract_address;
        let mut suffix = "".to_string();
        let mut salt;
        let mut deployed = false;
        loop {
            let user = external_user_id.to_string().clone() + suffix.as_str();
            salt = get_hash(user);
            contract_address =
                SimpleAccountFactoryProvider::get_address(provider, user_wallet, salt).await?;
            if contract_exists_at(format!("{:?}", contract_address)).await {
                info!("contract exists at {:?}", contract_address);
                if Self::is_deployed_by_us(provider, contract_address).await? {
                    deployed = true;
                    break;
                }
            } else {
                break;
            }
            suffix = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_err(|err| ProviderError(format!("Failed to create suffix: {:?}", err)))?
                .as_nanos()
                .to_string();
        }
        Ok(Wallet {
            address: contract_address,
            salt: BigDecimal::from(salt),
            deployed,
        })
    }

    async fn is_deployed_by_us(
        provider: &Web3Client,
        contract_address: Address,
    ) -> Result<bool, ProviderError> {
        let account_deployed_by =
            SimpleAccountProvider::get_deployer(provider, contract_address).await;
        match account_deployed_by {
            Ok(account_deployed_by) => {
                Ok(account_deployed_by == CONFIG.run_config.deployed_by_identifier)
            }
            Err(err) => Err(err),
        }
    }

    pub async fn list_transactions(
        &self,
        page_size: i64,
        id: Option<i32>,
        user: User,
    ) -> Vec<Transaction> {
        let row_id = id.unwrap_or(i32::MAX);

        let mut transactions = Vec::new();
        let result = self
            .transaction_dao
            .list_transactions(page_size, row_id, user.wallet_address)
            .await;

        for transaction_and_exponent in result {
            transactions.push(Transaction::from(transaction_and_exponent))
        }

        transactions
    }
}

struct Wallet {
    pub address: Address,
    pub salt: BigDecimal,
    pub deployed: bool,
}
