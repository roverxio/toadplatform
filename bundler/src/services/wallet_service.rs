use std::sync::Arc;
use std::time::SystemTime;

use ethers::providers::{Http, Provider};
use ethers::types::Address;
use log::{info, warn};

use crate::contracts::simple_account_factory_provider::SimpleAccountFactory;
use crate::contracts::simple_account_provider::SimpleAccountProvider;
use crate::db::dao::transaction_dao::TransactionDao;
use crate::db::dao::wallet_dao::WalletDao;
use crate::errors::ApiError;
use crate::models::transaction::transaction::Transaction;
use crate::models::wallet::address_response::AddressResponse;
use crate::provider::helpers::{contract_exists_at, get_hash};
use crate::CONFIG;

#[derive(Clone)]
pub struct WalletService {
    pub wallet_dao: WalletDao,
    pub transaction_dao: TransactionDao,
    pub simple_account_factory_provider: SimpleAccountFactory<Provider<Http>>,
    pub client: Arc<Provider<Http>>,
}

impl WalletService {
    pub async fn get_wallet_address(&self, usr: &str) -> Result<AddressResponse, ApiError> {
        let result: Wallet;
        let address = self.wallet_dao.get_wallet_address(usr.to_string()).await;
        if address.is_empty() {
            result = self.get_address(usr).await;
            info!("salt -> {}", result.salt);
            self.wallet_dao
                .create_wallet(
                    usr.to_string(),
                    format!("{:?}", result.address),
                    result.salt.to_string(),
                    false,
                )
                .await;
        } else {
            result = Wallet {
                address: address.parse().unwrap(),
                salt: "".to_string(),
            }
        }

        Ok(AddressResponse {
            address: result.address,
        })
    }

    async fn get_address(&self, usr: &str) -> Wallet {
        let mut result;
        let mut suffix = "".to_string();
        let mut salt;
        loop {
            let user = usr.to_string().clone() + suffix.as_str();
            salt = get_hash(user).to_string().parse().unwrap();
            result = self
                .simple_account_factory_provider
                .get_address(CONFIG.run_config.account_owner, salt)
                .await
                .unwrap();
            if contract_exists_at(format!("{:?}", result)).await {
                info!("contract exists at {:?}", result);
                if self.is_deployed_by_us(result).await {
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
        Wallet {
            address: result,
            salt: salt.to_string(),
        }
    }

    async fn is_deployed_by_us(&self, contract_address: Address) -> bool {
        let simple_account_provider =
            SimpleAccountProvider::init_abi(self.client.clone(), contract_address);
        let account_deployed_by = simple_account_provider.deployed_by().call().await;
        match account_deployed_by {
            Ok(account_deployed_by) => {
                account_deployed_by == CONFIG.run_config.deployed_by_identifier.clone()
            }
            Err(err) => {
                warn!("Error while calling 'deployedBy' -> {}", err);
                false
            }
        }
    }

    pub async fn list_transactions(
        &self,
        page_size: i32,
        id: Option<i32>,
        user_id: &String,
    ) -> Vec<Transaction> {
        let user_wallet_address = self
            .wallet_dao
            .get_wallet_address(user_id.to_string())
            .await;

        let row_id = id.unwrap_or(i32::MAX);

        let mut transactions = Vec::new();
        let result = self
            .transaction_dao
            .list_transactions(page_size, row_id, user_wallet_address)
            .await;

        for transaction_and_exponent in result {
            transactions.push(Transaction::from(transaction_and_exponent))
        }

        transactions
    }
}

struct Wallet {
    pub address: Address,
    pub salt: String,
}
