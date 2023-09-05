use crate::sync_transfers::{sync_user_transactions, Table};
use lazy_static::lazy_static;

pub mod config;
pub mod constants;
pub mod sync_transfers;
pub mod utils;

lazy_static! {
    // define CONFIG and POOL
}

fn main() {
    // read command line arguments
    sync_user_transactions(Table::from(""));
}
