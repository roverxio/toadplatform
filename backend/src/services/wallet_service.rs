use ethers::providers::{Http, Provider};
use ethers::types::{Address, U256};
use crate::CONFIG;

use crate::db::dao::wallet_dao::WalletDao;
use crate::errors::ApiError;
use crate::models::wallet::address_response::AddressResponse;
use crate::provider::helpers::get_hash;
use crate::provider::web3_provider::SimpleAccountFactory;

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
            let salt: U256 = get_hash(usr.to_string()).to_string().parse().unwrap();
            result = self.simple_account_factory_provider.get_address(CONFIG.account_owner, salt).await.unwrap();
        } else {
            result = address.parse().unwrap();
        }

        Ok(AddressResponse {
            address: result,
        })
    }
}
