use lazy_static::lazy_static;
use std::env::args;

use crate::db::connection::Connection;
use crate::db::token_transfers::TokenTransfers;
use crate::db::transactions::Transactions;
use crate::db::user_transactions::UserTransaction;
use crate::settings::Settings;
use crate::utils::table::Table;
use crate::utils::utils::Utils;

pub mod db;
pub mod settings;
pub mod utils;

lazy_static! {
    static ref CONFIG: Settings = Settings::new().expect("Unable to import config");
}

fn main() {
    let _pool = Connection::init();

    let table = Table::from(args().nth(1).expect("no table given"));

    let _last_sync_time = Utils::get_last_synced_time(table.clone());

    let user_transactions = match table {
        Table::TokenTransfers => {
            UserTransaction::from_token_transfers(TokenTransfers::get(_last_sync_time))
        }
        Table::Transactions => {
            UserTransaction::from_transactions(Transactions::get(_last_sync_time))
        }
    };

    UserTransaction::insert(user_transactions);
}
