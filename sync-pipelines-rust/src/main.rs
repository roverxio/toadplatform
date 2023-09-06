use lazy_static::lazy_static;

use crate::config::Config;
use crate::db::connection::Connection;
use crate::sync_transfers::sync_user_transactions;
use crate::utils::table::Table;

pub mod config;
pub mod db;
pub mod sync_transfers;
pub mod utils;

lazy_static! {
    static ref CONFIG: Config = Config::new();
}

fn main() {
    let pool = Connection::init();
    // read command line arguments
    sync_user_transactions(Table::from(""), &pool);
}
