use config::{Config, ConfigError, File, Map};
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
pub struct DefaultChain {
    pub chain: String,
    pub currency: String
}

#[derive(Debug, Deserialize, Clone)]
pub struct Chain {
    pub url: String,
    pub currency: String,
}

impl Chain {
    pub fn get_url(&self) -> String {
        format!("{}{}", self.url.clone(), std::env::var("INFURA_KEY").unwrap())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub current_chain: String,
    pub log: Log,
    pub database: Database,
    pub server: Server,
    pub default_chain: DefaultChain,
    pub chains: Map<String, Chain>,
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
