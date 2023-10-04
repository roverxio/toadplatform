use crate::db::token_transfers::TokenTransfers;
use crate::db::transactions::Transactions;
use crate::db::user_transactions::UserTransaction;
use crate::models::last_sync::LastSync;
use crate::utils::table::Table;
use sqlx::{Pool, Postgres};
use std::process::exit;

pub struct SyncUserTransactions {}

impl SyncUserTransactions {
    pub async fn sync_from_token_transfers(pool: Pool<Postgres>) -> Result<(), String> {
        let block_number = LastSync::get_last_synced_block(Table::TokenTransfers);
        let token_transfers = TokenTransfers::get(pool.clone(), block_number).await?;
        if token_transfers.len() == 0 {
            exit(0);
        }
        let number = token_transfers[0].block_number.unwrap_or(0);
        UserTransaction::insert(pool, UserTransaction::from_token_transfers(token_transfers))
            .await?;
        LastSync::update_last_synced_block(Table::TokenTransfers, number)
    }

    pub async fn sync_from_transactions(pool: Pool<Postgres>) -> Result<(), String> {
        let block_number = LastSync::get_last_synced_block(Table::Transactions);
        let transactions = Transactions::get(pool.clone(), block_number).await?;
        if transactions.len() == 0 {
            exit(0);
        }
        let number = transactions[0].block_number.unwrap_or(0);
        UserTransaction::insert(pool, UserTransaction::from_transactions(transactions)).await?;
        LastSync::update_last_synced_block(Table::Transactions, number)
    }
}
