use serde::Serialize;

#[derive(Serialize)]
pub struct BalanceResponse {
    pub address: String,
    pub balance: String,
    pub currency: String,
}

impl BalanceResponse {
    pub fn new(balance: String, address: String, currency: String) -> BalanceResponse {
        BalanceResponse {
            balance,
            address,
            currency,
        }
    }
}
