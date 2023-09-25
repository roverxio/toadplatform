use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Error {
    #[serde(skip_serializing_if = "String::is_empty")]
    message: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub data: Value,
    pub err: Error,
}

impl From<String> for ErrorResponse {
    fn from(error: String) -> Self {
        ErrorResponse {
            data: json!({}),
            err: Error { message: error },
        }
    }
}
