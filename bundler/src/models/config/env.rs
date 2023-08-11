use std::fmt;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub enum ENV {
    Development,
    Staging,
    Production,
}

impl fmt::Display for ENV {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ENV::Development => write!(f, "development"),
            ENV::Staging => write!(f, "staging"),
            ENV::Production => write!(f, "production"),
        }
    }
}
