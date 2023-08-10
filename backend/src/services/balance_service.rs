use crate::db::dao::wallet_dao::WalletDao;
use crate::errors::ApiError;
use crate::models::wallet::balance_response::BalanceResponse;
use crate::provider::web3_provider::ERC20;
use crate::PROVIDER;
use ethers::abi::Address;
use ethers::providers::{Http, Middleware, Provider};

#[derive(Clone)]
pub struct BalanceService {
    pub wallet_dao: WalletDao,
    pub erc20_provider: ERC20<Provider<Http>>,
}

impl BalanceService {
    pub async fn get_wallet_balance(
        &self,
        chain: &String,
        currency: &String,
        user: &str,
    ) -> Result<BalanceResponse, ApiError> {
        println!("Chain: {:?}", chain); // will be relevant when we add support for multiple chains
        let mut balance: String = "0".to_string();
        let address = self.wallet_dao.get_wallet_address(user.to_string()).await;
        if address.is_empty() {
            return Err(ApiError::NotFound("Wallet not found".to_string()));
        }
        let user: Address = address.parse().unwrap();
        if currency == "native" {
            balance = PROVIDER
                .get_balance(user.clone(), None)
                .await
                .unwrap()
                .to_string();
            balance = (balance.parse::<f64>().unwrap() / 1e18).to_string();
        } else if currency == "usdc" {
            let usdc_balance = self
                .erc20_provider
                .balance_of(user.clone())
                .await
                .unwrap()
                .to_string();
            balance = (usdc_balance.parse::<f64>().unwrap() / 1e6).to_string();
        }
        Ok(BalanceResponse {
            balance: balance.clone(),
            address: address.clone(),
            currency: currency.to_string(),
        })
    }
}
