pub enum Currency {
    Native,
    Erc20,
}

impl Currency {
    pub fn from_str(s: String) -> Option<Currency> {
        match s.to_lowercase().as_str() {
            "native" => Some(Currency::Native),
            "erc20" => Some(Currency::Erc20),
            _ => None,
        }
    }
}
