use config::{Config, ConfigError, File, Map};
use ethers::types::{Address};
use serde::Deserialize;

use crate::models::config::env::ENV;

#[derive(Debug, Deserialize, Clone)]
pub struct Log {
    pub level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub port: u16,
    pub host: String,
    pub prefix: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Database {
    pub file: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Urls {
    pub signing_server: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DefaultChain {
    pub chain: String,
    pub currency: String
}

#[derive(Debug, Deserialize, Clone)]
pub struct Chain {
    pub chain_id: u64,
    pub url: String,
    pub simple_account_factory_address: Address,
    pub usdc_address: Address,
    pub entrypoint_address: Address,
    pub verifying_paymaster_address: Address,
}

impl Chain {
    pub fn get_url(&self) -> String {
        format!("{}{}", self.url.clone(), std::env::var("INFURA_KEY").expect("INFURA_KEY must be set"))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct DefaultGas {
    pub call_gas_limit: u64,
    pub verification_gas_limit: u64,
    pub pre_verification_gas: u64,
    pub max_fee_per_gas: u64,
    pub max_priority_fee_per_gas: u64
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub current_chain: String,
    pub account_owner: Address,
    pub paymaster_account_owner: Address,
    pub log: Log,
    pub database: Database,
    pub urls: Urls,
    pub server: Server,
    pub default_chain: DefaultChain,
    pub chains: Map<String, Chain>,
    pub default_gas: DefaultGas,
    pub env: ENV,
}

const CONFIG_FILE_PATH: &str = "./config/Development.toml";
const CONFIG_FILE_PREFIX: &str = "./config";

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let env = std::env::var("RUN_ENV").unwrap_or_else(|_| "Development".into());

        let s = Config::builder()
            .add_source(File::with_name(CONFIG_FILE_PATH).required(false))
            .add_source(File::with_name(&format!("{}/{}", CONFIG_FILE_PREFIX, env)).required(false))
            .set_override("env", env)?
            .build()?;

        s.try_deserialize()
    }
}
