use serde::Serialize;

#[derive(Serialize)]
pub struct AddressResponse {
    pub address: String,
}
