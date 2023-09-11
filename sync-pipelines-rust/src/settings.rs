use std::fs::File;
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use config::{Config, ConfigError};

pub struct LastSyncTime {
    start_time: StartTime,
    sync_files: SyncFiles,
}

pub struct SyncFiles {
    token_transfers: PathBuf,
    transactions: PathBuf,
}

pub struct StartTime {
    token_transfers: DateTime<Utc>,
    transactions: DateTime<Utc>,
}

pub struct Settings {
    chain: String,
    last_sync_time: LastSyncTime,
    native_currency: String,
    transaction_id_prefix: String,
}

const CONFIG_FILE_PATH: &str = "../Config.toml";

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name(CONFIG_FILE_PATH).required(false))
            .build()?;

        s.try_deserialize()
    }

    pub fn get_last_sync_file_token_transfers(&self) -> &PathBuf {
        &self.last_sync_time.sync_files.token_transfers
    }

    pub fn get_last_sync_file_transactions(&self) -> &PathBuf {
        &self.last_sync_time.sync_files.transactions
    }

    pub fn get_last_sync_time_token_transfers(&self) -> DateTime<Utc> {
        self.last_sync_time.start_time.token_transfers
    }

    pub fn get_last_sync_time_transactions(&self) -> DateTime<Utc> {
        self.last_sync_time.start_time.token_transfers
    }

    pub fn get_chain(&self) -> &String {
        &self.chain
    }

    pub fn get_native_currency(&self) -> &String {
        &self.native_currency
    }

    pub fn get_transaction_id_prefix(&self) -> &String {
        &self.transaction_id_prefix
    }
}
