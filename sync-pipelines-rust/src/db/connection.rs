use sqlx::{Pool, Postgres};

pub struct Connection {}

impl Connection {
    pub fn init() -> Pool<Postgres> {
        unimplemented!()
    }
}
