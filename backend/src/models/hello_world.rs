use serde::{Deserialize, Serialize};

#[derive(PartialEq,Debug)]
#[derive(Serialize, Deserialize)]
pub struct HelloWorld {
    pub name: String,
}

