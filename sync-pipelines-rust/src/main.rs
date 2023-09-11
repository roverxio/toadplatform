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

    let user_transactions = match table_name {
        Table::TokenTransfers => UserTransaction::from_token_transfers(TokenTransfers::get(pool)),
        Table::Transactions => UserTransaction::from_transactions(Transactions::get(pool)),
    };

    UserTransaction::insert(user_transactions);
}
