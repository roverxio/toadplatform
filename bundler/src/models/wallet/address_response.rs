use ethers::types::Address;
use serde::Serialize;

#[derive(Serialize)]
pub struct AddressResponse {
    pub address: Address,
}
