use serde::Deserialize;

#[derive(Deserialize)]
pub struct ListTransactions {
    pub id: Option<i32>,
    pub page_size: i32,
}
