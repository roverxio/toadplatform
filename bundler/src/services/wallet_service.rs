use actix_web::rt::spawn;
use bigdecimal::{BigDecimal, Zero};
use ethers::types::Address;
use log::info;
use sqlx::{Pool, Postgres};
use std::time::SystemTime;

use crate::contracts::simple_account_factory_provider::SimpleAccountFactoryProvider;
use crate::contracts::simple_account_provider::SimpleAccountProvider;
use crate::db::dao::{TransactionDao, User, WalletDao};
use crate::errors::{ProviderError, TransactionError, WalletError};
use crate::models::transaction::Transaction;
use crate::models::wallet::AddressResponse;
use crate::provider::helpers::{contract_exists_at, get_hash};
use crate::provider::Web3Client;
use crate::services::MintService;
use crate::CONFIG;

#[derive(Clone)]
pub struct WalletService;

impl WalletService {
    pub async fn get_wallet_address(
        pool: &Pool<Postgres>,
        provider: &Web3Client,
        user: User,
        user_wallet: String,
    ) -> Result<AddressResponse, WalletError> {
        let result: Wallet;
        if user.wallet_address.is_empty() {
            result = Self::get_address(
                &provider.clone(),
                user.external_user_id.as_str(),
                user_wallet.parse().unwrap(),
            )
            .await?;
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
            .await?;
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
        pool: &Pool<Postgres>,
        page_size: i64,
        id: Option<i32>,
        user: User,
    ) -> Result<Vec<Transaction>, TransactionError> {
        let row_id = id.unwrap_or(i32::MAX);

        let result =
            TransactionDao::list_transactions(pool, page_size, row_id, user.wallet_address).await?;

        let transactions = result
            .iter()
            .map(|txn| Transaction::from(txn.clone()))
            .collect();

        Ok(transactions)
    }
}

struct Wallet {
    pub address: Address,
    pub salt: BigDecimal,
    pub deployed: bool,
}
