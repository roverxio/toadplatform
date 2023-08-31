use serde::Deserialize;

#[derive(Deserialize)]
pub struct ListTransactionsParams {
    pub id: Option<i32>,
    pub page_size: Option<i64>,
}
