use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub fn establish_connection(database_url: String) -> Pool<SqliteConnectionManager> {
    let manager = SqliteConnectionManager::file(&database_url);
    Pool::new(manager).unwrap()
}
