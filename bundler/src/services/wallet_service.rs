use std::time::SystemTime;

use ethers::providers::{Http, Provider};
use ethers::types::{Address, U256};
use log::{debug, info};

use crate::db::dao::wallet_dao::WalletDao;
use crate::errors::ApiError;
use crate::models::transaction::transaction::{Amount, Metadata, Transaction, UserInfo};
use crate::models::wallet::address_response::AddressResponse;
use crate::provider::helpers::{contract_exists_at, get_hash};
use crate::provider::web3_provider::SimpleAccountFactory;
use crate::CONFIG;

#[derive(Clone)]
pub struct WalletService {
    pub wallet_dao: WalletDao,
    pub simple_account_factory_provider: SimpleAccountFactory<Provider<Http>>,
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
        let mut contract_exists = true;
        let mut result: Address = Default::default();
        let mut suffix = "".to_string();
        let mut salt = U256::zero();
        while contract_exists {
            let user = usr.to_string().clone() + suffix.as_str();
            salt = get_hash(user).to_string().parse().unwrap();
            result = self
                .simple_account_factory_provider
                .get_address(CONFIG.run_config.account_owner, salt)
                .await
                .unwrap();
            if !contract_exists_at(format!("{:?}", result)).await {
                contract_exists = false;
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

    pub fn list_transactions(&self, page_size: i32, id: Option<i32>) -> Vec<Transaction> {
        debug!("page_size -> {}", page_size);
        debug!("id -> {:?}", id);
        let mut transactions = Vec::new();
        transactions.push(Transaction {
            transaction_id: "txn_id_1".to_string(),
            amount: Amount {
                currency: "usdc".to_string(),
                value: "10000000".to_string(),
                exponent: "6".to_string(),
            },
            metadata: Metadata {
                chain: CONFIG.run_config.current_chain.clone(),
                gas: Amount {
                    currency: CONFIG.chains[&CONFIG.run_config.current_chain]
                        .currency
                        .clone(),
                    value: "100000000000".to_string(),
                    exponent: "18".to_string(),
                },
                transaction_hash: "0xtransaction_hash".to_string(),
                timestamp: "2023-05-12T16:41:45.530002+00".to_string(),
                explorer_url: "https://www.example.com".to_string(),
                status: "pending".to_string(),
            },
            from: UserInfo {
                address: "0xfrom_address".to_string(),
                name: "".to_string(),
            },
            id: 2,
            to: UserInfo {
                address: "0xto_address".to_string(),
                name: "a toad user".to_string(),
            },
            transaction_type: "credit".to_string(),
        });
        transactions.push(Transaction {
            transaction_id: "txn_id_1".to_string(),
            amount: Amount {
                currency: "usdc".to_string(),
                value: "1000000".to_string(),
                exponent: "6".to_string(),
            },
            metadata: Metadata {
                chain: CONFIG.run_config.current_chain.clone(),
                gas: Amount {
                    currency: CONFIG.chains[&CONFIG.run_config.current_chain]
                        .currency
                        .clone(),
                    value: "800000000000".to_string(),
                    exponent: "18".to_string(),
                },
                transaction_hash: "0xtransaction_hash".to_string(),
                timestamp: "2023-05-11T16:41:45.530002+00".to_string(),
                explorer_url: "https://www.example.com".to_string(),
                status: "pending".to_string(),
            },
            from: UserInfo {
                address: "0xfrom_address".to_string(),
                name: "a toad user".to_string(),
            },
            id: 1,
            to: UserInfo {
                address: "0xto_address".to_string(),
                name: "".to_string(),
            },
            transaction_type: "debit".to_string(),
        });
        transactions
    }
}

struct Wallet {
    pub address: Address,
    pub salt: String,
}
