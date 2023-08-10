use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct HelloWorld {
    pub name: String,
}
