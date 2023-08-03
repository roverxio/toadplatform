use crate::errors::ApiError;
use crate::models::wallet::balance_response::BalanceResponse;

#[derive(Clone)]
pub struct BalanceService {}

impl BalanceService {
    pub fn get_wallet_balance(&self, chain: &String, currency: &String) -> Result<BalanceResponse, ApiError> {
        println!("Chain: {:?}", chain);
        println!("Currency: {:?}", currency);
        Ok(BalanceResponse {
            balance: "0.0287".to_string(),
            address: "0x773C77D66D831dF29097c1604947F5b8fb0667A4".to_string(),
            currency: currency.to_string()
        })
    }
}
