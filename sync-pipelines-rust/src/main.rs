use lazy_static::lazy_static;
use std::env::args;

use crate::db::connection::Connection;
use crate::db::token_transfers::TokenTransfers;
use crate::db::transactions::Transactions;
use crate::db::user_transactions::UserTransaction;
use crate::settings::Settings;
use crate::utils::table::Table;

pub mod db;
pub mod settings;
pub mod utils;

lazy_static! {
    static ref CONFIG: Settings = Settings::new().expect("Unable to import config");
}

#[tokio::main]
async fn main() {
    let pool = Connection::init().await;

    let table_name = Table::from(args().nth(1).expect("no table given"));

    println!(
        " {}\n {}\n {}\n {}\n {}\n {}\n, {}\n",
        CONFIG.get_native_currency(),
        CONFIG.get_chain(),
        CONFIG.get_transaction_id_prefix(),
        CONFIG.get_last_sync_time_transactions(),
        CONFIG.get_last_sync_block_token_transfers(),
        CONFIG.get_last_sync_file_transactions().to_string_lossy(),
        CONFIG
            .get_last_sync_file_token_transfers()
            .to_string_lossy()
    );

    let user_transactions = match table_name {
        Table::TokenTransfers => UserTransaction::from_token_transfers(TokenTransfers::get(pool)),
        Table::Transactions => UserTransaction::from_transactions(Transactions::get(pool)),
    };

    UserTransaction::insert(user_transactions);
}
