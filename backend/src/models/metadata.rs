use serde::Deserialize;

#[derive(Deserialize)]
pub struct Metadata {
    pub currency: String,
    pub chain: String,
}
