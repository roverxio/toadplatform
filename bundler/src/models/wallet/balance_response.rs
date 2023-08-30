use serde::Serialize;

#[derive(Serialize)]
pub struct BalanceResponse {
    pub address: String,
    pub balance: String,
    pub currency: String,
    pub exponent: i32,
}

impl BalanceResponse {
    pub fn new(
        balance: String,
        address: String,
        currency: String,
        exponent: i32,
    ) -> BalanceResponse {
        BalanceResponse {
            balance,
            address,
            currency,
            exponent,
        }
    }
}
