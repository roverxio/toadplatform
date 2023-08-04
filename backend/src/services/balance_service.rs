use ethers::abi::Address;
use ethers::providers::Middleware;
use crate::db::dao::wallet_dao::WalletDao;
use crate::errors::ApiError;
use crate::models::wallet::balance_response::BalanceResponse;
use crate::PROVIDER;

#[derive(Clone)]
pub struct BalanceService {
    pub wallet_dao: WalletDao,
}

impl BalanceService {
    pub async fn get_wallet_balance(&self, chain: &String, currency: &String, user: &str) -> Result<BalanceResponse, ApiError> {
        println!("Chain: {:?}", chain); // will be relevant when we add support for multiple chains
        let mut balance: String = "0".to_string();
        let address = self.wallet_dao.get_wallet(user.to_string()).await;
        if currency == "native" {
            if address.is_empty() {
                return Err(ApiError::NotFound("Wallet not found".to_string()));
            }
            let user: Address = address.parse().unwrap();
            balance = PROVIDER.get_balance(user.clone(), None).await.unwrap().to_string();
            // balance from wei to ether
            balance = (balance.parse::<f64>().unwrap() / 1e18).to_string();
        } else if currency == "usdc" {
            // get usdc balance of user
        }
        Ok(BalanceResponse {
            balance: balance.clone(),
            address: address.clone(),
            currency: currency.to_string(),
        })
    }
}
