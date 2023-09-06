use std::process::exit;

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

    pub fn from(s: &str) -> Self {
        match s {
            "token_transfers" => Table::TokenTransfers,
            "transactions" => Table::Transactions,
            _ => {
                // raise error
                exit(1);
            }
        }
    }
}
