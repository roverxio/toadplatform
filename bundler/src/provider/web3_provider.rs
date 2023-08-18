use ethers::providers::{Http, Middleware, Provider};
use ethers::types::Address;
use log::error;

use crate::PROVIDER;

#[derive(Clone)]
pub struct Web3Provider {}

impl Web3Provider {
    pub fn new(chain_url: String) -> Provider<Http> {
        let provider = Provider::try_from(chain_url).unwrap();
        provider
    }

    pub async fn get_balance(address: Address) -> Result<String, String> {
        let result = PROVIDER.get_balance(address, None).await;
        if result.is_err() {
            error!("Get native balance failed: {:?}", result.err().unwrap());
            return Err(String::from("Failed to get balance"));
        }
        let wei_balance = result.unwrap().to_string();
        if wei_balance.parse::<f64>().is_err() {
            return Err(String::from("Failed to parse balance"));
        }
        Ok((wei_balance.parse::<f64>().unwrap() / 1e18).to_string())
    }
}
