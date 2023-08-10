use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use actix_web::{error, web};

pub async fn connect(pool: Pool<SqliteConnectionManager>) -> PooledConnection<SqliteConnectionManager> {
    let conn = web::block(move || pool.get()).await.unwrap().map_err(error::ErrorInternalServerError).unwrap(); // <- create async connection (non-blocking
    return conn;
}
