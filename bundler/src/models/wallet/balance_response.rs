use serde::Serialize;

#[derive(Serialize)]
pub struct BalanceResponse {
    pub address: String,
    pub balance: String,
    pub currency: String,
    pub exponent: u8,
}

impl BalanceResponse {
    pub fn new(
        balance: String,
        address: String,
        currency: String,
        exponent: u8,
    ) -> BalanceResponse {
        BalanceResponse {
            balance,
            address,
            currency,
            exponent,
        }
    }
}
