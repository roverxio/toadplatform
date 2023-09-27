use config::{Config, ConfigError, File, Map};
use ethers::types::Address;
use serde::Deserialize;

use crate::models::config::env::ENV;

#[derive(Debug, Deserialize, Clone)]
pub struct RunConfig {
    pub current_chain: String,
    pub default_currency: String,
    pub account_owner: Address,
    pub paymaster_account_owner: Address,
    pub deployed_by_identifier: String,
    pub transaction_id_prefix: String,
}

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
pub struct Chain {
    pub chain_id: u64,
    pub url: String,
    pub explorer_url: String,
    pub simple_account_factory_address: Address,
    pub usdc_address: Address,
    pub currency: String,
    pub entrypoint_address: Address,
    pub verifying_paymaster_address: Address,
}

impl Server {
    pub fn get_port(&self) -> u16 {
        let port = std::env::var("PORT");
        if port.is_ok() {
            port.unwrap().parse::<u16>().unwrap()
        } else {
            self.port
        }
    }
}

impl Chain {
    pub fn get_url(&self) -> String {
        format!(
            "{}{}",
            self.url.clone(),
            std::env::var("PROVIDER_API_KEY")
                .expect("PROVIDER_API_KEY must be set to connect with a node provider")
        )
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct DefaultGas {
    pub call_gas_limit: u64,
    pub verification_gas_limit: u64,
    pub pre_verification_gas: u64,
    pub max_fee_per_gas: u64,
    pub max_priority_fee_per_gas: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub run_config: RunConfig,
    pub log: Log,
    pub server: Server,
    pub chains: Map<String, Chain>,
    pub default_gas: DefaultGas,
    pub admins: Vec<String>,
    pub env: ENV,
}

const CONFIG_FILE_PATH: &str = "./config/Development.toml";
const CONFIG_FILE_PREFIX: &str = "./config";

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let env = std::env::var("RUN_ENV").unwrap_or_else(|_| "Development".into());
        let admins_env = std::env::var("ADMIN").expect("ADMIN env variable not set");
        let admins = admins_env
            .split(",")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let s = Config::builder()
            .add_source(File::with_name(CONFIG_FILE_PATH).required(false))
            .add_source(File::with_name(&format!("{}/{}", CONFIG_FILE_PREFIX, env)).required(false))
            .set_override("env", env)?
            .set_override("admins", admins)?
            .build()?;

        s.try_deserialize()
    }

    pub fn get_chain(&self) -> &Chain {
        &self.chains[&self.run_config.current_chain]
    }

    pub fn get_admins(&self) -> &Vec<String> {
        &self.admins
    }
}
