use chrono::{DateTime, Utc};
use std::path::PathBuf;

pub struct LastSyncTime {
    _transactions: PathBuf,
    _token_transfers: PathBuf,
    _start_time: StartTime,
}

pub struct StartTime {
    _transactions: DateTime<Utc>,
    _token_transfers: DateTime<Utc>,
}

pub struct ERC20Contracts {
    _address: String,
    _symbol: String,
}

pub struct Config {
    _last_sync_time: LastSyncTime,
    _erc20_contracts: Vec<ERC20Contracts>,
    _transaction_id_prefix: String,
    _native_currency: String,
    _chain: String,
}

impl Config {
    pub fn new() -> Self {
        unimplemented!();
    }

    // getters
}
