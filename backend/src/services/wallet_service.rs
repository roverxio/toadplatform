use std::time::SystemTime;

use ethers::providers::{Http, Provider};
use ethers::types::{Address, U256};
use log::info;

use crate::db::dao::wallet_dao::WalletDao;
use crate::errors::ApiError;
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
                .get_address(CONFIG.account_owner, salt)
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
}

struct Wallet {
    pub address: Address,
    pub salt: String,
}
