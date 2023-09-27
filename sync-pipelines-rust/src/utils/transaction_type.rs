pub enum TransactionType {
    CREDIT,
}

impl TransactionType {
    pub fn to_string(&self) -> String {
        match self {
            TransactionType::CREDIT => String::from("credit"),
        }
    }
}
