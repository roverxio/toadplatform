use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
pub struct LastSyncBlock {
    block: Block,
    files: Files,
}

#[derive(Deserialize)]
pub struct Files {
    token_transfers: PathBuf,
    transactions: PathBuf,
}

#[derive(Deserialize)]
pub struct Block {
    token_transfers: i64,
    transactions: i64,
}

#[derive(Deserialize)]
pub struct Settings {
    chain: String,
    last_sync_block: LastSyncBlock,
    native_currency: String,
    transaction_id_prefix: String,
}

const CONFIG_FILE_PATH: &str = "./Config.toml";

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name(CONFIG_FILE_PATH).required(true))
            .build()?;

        s.try_deserialize()
    }

    pub fn get_last_sync_file_token_transfers(&self) -> &Path {
        &self.last_sync_block.files.token_transfers
    }

    pub fn get_last_sync_file_transactions(&self) -> &Path {
        &self.last_sync_block.files.transactions
    }

    pub fn get_last_sync_block_token_transfers(&self) -> i64 {
        self.last_sync_block.block.token_transfers.clone()
    }

    pub fn get_last_sync_block_transactions(&self) -> i64 {
        self.last_sync_block.block.transactions.clone()
    }

    pub fn get_chain(&self) -> &str {
        &self.chain
    }

    pub fn get_native_currency(&self) -> &str {
        &self.native_currency
    }

    pub fn get_transaction_id_prefix(&self) -> &str {
        &self.transaction_id_prefix
    }
}
