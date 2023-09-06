use sqlx::{Pool, Postgres};

use crate::db::token_transfers::TokenTransfers;
use crate::db::transactions::Transactions;
use crate::db::user_transactions::UserTransaction;
use crate::utils::table::Table;
use crate::utils::utils::Utils;

pub fn sync_user_transactions(from_table: Table, _pool: &Pool<Postgres>) {
    // get last_synced_time
    let _last_sync_time = Utils::get_last_synced_time(from_table.to_string());
    // sqlx query to get relevant transfers - using users(wallet_addresses) x token_transfers(from last synced time)
    let user_transactions = match from_table {
        Table::TokenTransfers => {
            UserTransaction::from_token_transfers(TokenTransfers::get(_last_sync_time))
        }
        Table::Transactions => {
            UserTransaction::from_transactions(Transactions::get(_last_sync_time))
        }
    };

    UserTransaction::insert(user_transactions);
}
