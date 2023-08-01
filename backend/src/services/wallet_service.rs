use crate::errors::ApiError;
use crate::models::wallet::address_response::AddressResponse;

#[derive(Clone)]
pub struct WalletService {}

impl WalletService {
    pub fn get_wallet_address(&self) -> Result<AddressResponse, ApiError> {
        Ok(AddressResponse {
            address: "0x773C77D66D831dF29097c1604947F5b8fb0667A4".to_string(),
        })
    }
}
