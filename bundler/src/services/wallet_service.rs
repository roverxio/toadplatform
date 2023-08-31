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
use crate::models::transaction::transaction::{Amount, Metadata, Transaction, UserInfo};
use crate::models::wallet::address_response::AddressResponse;
use crate::provider::helpers::{contract_exists_at, get_explorer_url, get_hash};
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
        page_size: i64,
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
            let transaction = transaction_and_exponent.user_transaction;
            transactions.push(Transaction {
                transaction_id: transaction.transaction_id,
                amount: Amount {
                    currency: transaction.currency,
                    value: transaction.amount,
                    exponent: transaction_and_exponent.exponent,
                },
                metadata: Metadata {
                    chain: transaction.metadata.chain,
                    gas: Amount::default(),
                    transaction_hash: transaction.metadata.transaction_hash.clone(),
                    timestamp: transaction.updated_at,
                    explorer_url: get_explorer_url(&transaction.metadata.transaction_hash),
                    status: transaction.status,
                },
                from: UserInfo {
                    address: transaction.from_address,
                    name: transaction.metadata.from_name,
                },
                id: transaction.id,
                to: UserInfo {
                    address: transaction.to_address,
                    name: transaction.metadata.to_name,
                },
                transaction_type: transaction.transaction_type,
            })
        }

        transactions
    }
}

struct Wallet {
    pub address: Address,
    pub salt: String,
}
