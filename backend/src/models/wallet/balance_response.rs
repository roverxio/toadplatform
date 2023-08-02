use serde::Serialize;

#[derive(Serialize)]
pub struct BalanceResponse {
    pub address: String,
    pub balance: String,
    pub currency: String
}
