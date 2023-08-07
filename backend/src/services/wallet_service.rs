use std::time::SystemTime;

use ethers::providers::{Http, Middleware, Provider};
use ethers::types::{Address, U256};

use crate::db::dao::wallet_dao::WalletDao;
use crate::errors::ApiError;
use crate::models::wallet::address_response::AddressResponse;
use crate::provider::helpers::get_hash;
use crate::provider::web3_provider::SimpleAccountFactory;
use crate::{CONFIG, PROVIDER};

#[derive(Clone)]
pub struct WalletService {
    pub wallet_dao: WalletDao,
    pub simple_account_factory_provider: SimpleAccountFactory<Provider<Http>>,
}

impl WalletService {
    pub async fn get_wallet_address(&self, usr: &str) -> Result<AddressResponse, ApiError> {
        let result: Address;
        let address = self.wallet_dao.get_wallet(usr.to_string()).await;
        if address.is_empty() {
            result = self.get_address(usr).await;
            self.wallet_dao
                .create_wallet(usr.to_string(), format!("{:?}", result))
                .await;
        } else {
            result = address.parse().unwrap();
        }

        Ok(AddressResponse { address: result })
    }

    async fn get_address(&self, usr: &str) -> Address {
        let mut contract_exists = true;
        let mut result: Address = Default::default();
        let mut suffix = "".to_string();
        while contract_exists {
            let user = usr.to_string().clone() + suffix.as_str();
            let salt: U256 = get_hash(user).to_string().parse().unwrap();
            result = self
                .simple_account_factory_provider
                .get_address(CONFIG.account_owner, salt)
                .await
                .unwrap();
            let code = PROVIDER.get_code(result, None).await.unwrap();
            if code.is_empty() {
                contract_exists = false;
            }
            suffix = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .to_string();
        }
        result
    }
}
