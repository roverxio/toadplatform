use crate::db::dao::wallet_dao::WalletDao;
use crate::errors::ApiError;
use crate::models::wallet::address_response::AddressResponse;
use crate::provider::helpers::get_hash;

#[derive(Clone)]
pub struct WalletService {
    pub wallet_dao: WalletDao,
}

impl WalletService {
    pub async fn get_wallet_address(&self, usr: &str) -> Result<AddressResponse, ApiError> {
        let address = self.wallet_dao.get_wallet(usr.to_string()).await;
        if address.is_empty() {
            println!("result: {}", get_hash(usr.to_string()));
        }
        Ok(AddressResponse {
            address,
        })
    }
}
