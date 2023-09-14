use log::error;
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
}
