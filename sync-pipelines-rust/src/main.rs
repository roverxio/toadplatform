use lazy_static::lazy_static;
use std::env::args;
use std::process::exit;

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

#[tokio::main]
async fn main() {
    let pool = Connection::init().await;

    let table_name = Table::from(args().nth(1).expect("No table argument provided"));

    match table_name {
        Table::TokenTransfers => {
            let block_number = Utils::get_last_synced_block_token_transfers();
            let token_transfers = TokenTransfers::get(pool.clone(), block_number).await;
            if token_transfers.clone().len() == 0 {
                exit(0);
            }
            let number = TokenTransfers::get_max_block_number(token_transfers.clone());
            UserTransaction::insert(pool, UserTransaction::from_token_transfers(token_transfers))
                .await;
            Utils::update_last_synced_block_token_transfers(number);
        }
        Table::Transactions => {
            let block_number = Utils::get_last_synced_block_transactions();
            let transactions = Transactions::get(pool.clone(), block_number).await;
            if transactions.clone().len() == 0 {
                exit(0);
            }
            let number = Transactions::get_max_block_timestamp(transactions.clone());
            UserTransaction::insert(pool, UserTransaction::from_transactions(transactions)).await;
            Utils::update_last_synced_block_transactions(number);
        }
    };
}
