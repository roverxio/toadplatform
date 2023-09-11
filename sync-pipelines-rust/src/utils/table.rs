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

    pub fn from(s: String) -> Self {
        match s {
            transfers if transfers == "token_transfers".to_string() => Table::TokenTransfers,
            transactions if transactions == "transactions".to_string() => Table::Transactions,
            _ => {
                error!("Invalid table argument");
                exit(1);
            }
        }
    }
}
