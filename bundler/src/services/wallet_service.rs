use actix_web::rt::spawn;
use bigdecimal::{BigDecimal, Zero};
use ethers::providers::{Http, Provider};
use ethers::types::Address;
use log::{info, warn};
use std::sync::Arc;
use std::time::SystemTime;

use crate::contracts::simple_account_factory_provider::{
    SimpleAccountFactory, SimpleAccountFactoryProvider,
};
use crate::contracts::simple_account_provider::SimpleAccountProvider;
use crate::db::dao::transaction_dao::TransactionDao;
use crate::db::dao::wallet_dao::{User, WalletDao};
use crate::errors::errors::ApiError;
use crate::models::transaction::transaction::Transaction;
use crate::models::wallet::address_response::AddressResponse;
use crate::provider::helpers::{contract_exists_at, get_hash};
use crate::services::mint_service::{mint, MintService};
use crate::CONFIG;

#[derive(Clone)]
pub struct WalletService {
    pub wallet_dao: WalletDao,
    pub transaction_dao: TransactionDao,
    pub simple_account_factory_provider: SimpleAccountFactory<Provider<Http>>,
    pub client: Arc<Provider<Http>>,
    pub mint_service: MintService,
}

impl WalletService {
    pub async fn get_wallet_address(
        &self,
        user: User,
        user_wallet: String,
    ) -> Result<AddressResponse, ApiError> {
        let result: Wallet;
        if user.wallet_address.is_empty() {
            result = self
                .get_address(user.external_user_id.as_str(), user_wallet.parse().unwrap())
                .await
                .unwrap();
            info!("salt -> {}", result.salt);
            WalletDao::create_wallet(
                &self.wallet_dao.pool.clone(),
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
            spawn(mint(
                result.address.clone(),
                self.mint_service.usdc_provider.clone(),
                self.mint_service.signer.clone(),
            ));
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
        &self,
        external_user_id: &str,
        user_wallet: Address,
    ) -> Result<Wallet, String> {
        let mut result;
        let mut suffix = "".to_string();
        let mut salt;
        let mut deployed = false;
        loop {
            let user = external_user_id.to_string().clone() + suffix.as_str();
            salt = get_hash(user);
            result = SimpleAccountFactoryProvider::get_address(
                self.simple_account_factory_provider.clone(),
                user_wallet,
                salt,
            )
            .await?;
            if contract_exists_at(format!("{:?}", result)).await {
                info!("contract exists at {:?}", result);
                if Self::is_deployed_by_us(self.client.clone(), result).await? {
                    deployed = true;
                    break;
                }
            } else {
                break;
            }
            suffix = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .to_string();
        }
        Ok(Wallet {
            address: result,
            salt: BigDecimal::from(salt),
            deployed,
        })
    }

    async fn is_deployed_by_us(
        client: Arc<Provider<Http>>,
        contract_address: Address,
    ) -> Result<bool, String> {
        let simple_account_provider = SimpleAccountProvider::init_abi(client, contract_address);
        let account_deployed_by = simple_account_provider.deployed_by().call().await;
        match account_deployed_by {
            Ok(account_deployed_by) => {
                Ok(account_deployed_by == CONFIG.run_config.deployed_by_identifier.clone())
            }
            Err(err) => {
                warn!("Error while calling 'deployedBy' -> {}", err);
                Err(format!("Error while calling 'deployedBy' -> {}", err))
            }
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
