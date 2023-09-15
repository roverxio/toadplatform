use lazy_static::lazy_static;
use log::error;
use std::env::args;
use std::process::exit;

use crate::db::connection::Connection;
use crate::services::sync_user_transactions::SyncUserTransactions;
use crate::settings::Settings;
use crate::utils::table::Table;

pub mod db;
pub mod models;
pub mod services;
pub mod settings;
pub mod utils;

lazy_static! {
    static ref CONFIG: Settings = Settings::new().expect("Unable to import config");
}

const LOG_CONFIG: &str = "./log_config.yaml";

#[tokio::main]
async fn main() {
    log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let table_arg = args().nth(1).unwrap_or_else(|| {
        error!("No table argument provided");
        exit(1)
    });

    let table_name = Table::from(table_arg).unwrap_or_else(|error| {
        error!("{error}");
        exit(1)
    });

    let pool = Connection::init().await.unwrap_or_else(|error| {
        error!("{error}");
        exit(1)
    });

    let result = match table_name {
        Table::TokenTransfers => SyncUserTransactions::sync_from_token_transfers(pool).await,
        Table::Transactions => SyncUserTransactions::sync_from_transactions(pool).await,
    };

    if result.is_err() {
        error!("{}", result.err().unwrap());
    }
}
