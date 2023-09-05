use sqlx::types::chrono::NaiveTime;
use std::path::PathBuf;

pub struct LastSyncTime {
    transactions: PathBuf,
    token_transfers: PathBuf,
}

pub struct StartTime {
    transactions: NaiveTime,
    token_transfers: NaiveTime,
}

pub struct ERC20Contracts {
    address: String,
    symbol: String,
}

pub struct Config {
    database_url: String,
    process_pool: u32,
    last_sync_time: LastSyncTime,
    start_time: StartTime,
    erc20_contracts: Vec<ERC20Contracts>,
    transaction_id_prefix: String,
    native_currency: String,
    chain: String,
}

impl Config {
    pub fn new() {
        // get config
        unimplemented!();
    }

    // getters
}
