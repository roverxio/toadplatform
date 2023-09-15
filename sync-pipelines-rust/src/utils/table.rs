use crate::CONFIG;
use log::error;
use std::path::Path;
use std::process::exit;

#[derive(Clone)]
pub enum Table {
    TokenTransfers,
    Transactions,
}

impl Table {
    pub fn to_string(&self) -> String {
        match self {
            Table::TokenTransfers => "token_transfers".to_string(),
            Table::Transactions => "transactions".to_string(),
        }
    }

    pub fn from(table: String) -> Self {
        match table.as_str() {
            "token_transfers" => Table::TokenTransfers,
            "transactions" => Table::Transactions,
            _ => {
                error!("Invalid table argument");
                exit(1);
            }
        }
    }

    pub fn get_path(&self) -> &Path {
        return match self {
            Table::TokenTransfers => CONFIG.get_last_sync_file_token_transfers(),
            Table::Transactions => CONFIG.get_last_sync_file_transactions(),
        };
    }

    pub fn get_block_number(&self) -> i64 {
        return match self {
            Table::TokenTransfers => CONFIG.get_last_sync_block_token_transfers(),
            Table::Transactions => CONFIG.get_last_sync_block_transactions(),
        };
    }
}
