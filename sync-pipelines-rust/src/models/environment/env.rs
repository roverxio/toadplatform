pub enum Constants {
    Development,
    Production,
    Staging,
}

impl Constants {
    pub fn to_string(&self) -> String {
        match self {
            Constants::Development => "development".to_string(),
            Constants::Production => "production".to_string(),
            Constants::Staging => "staging".to_string(),
        }
    }
}
