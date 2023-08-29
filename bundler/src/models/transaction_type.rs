pub enum TransactionType {
    Debit,
}

impl TransactionType {
    pub fn to_string(&self) -> String {
        match self {
            Self::Debit => String::from("debit"),
        }
    }
}
