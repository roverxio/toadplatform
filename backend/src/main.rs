use crate::models::config::server::Server;
use crate::server::{api_server, init_services};

mod handlers;
mod routes;
mod errors;
mod services;
mod models;
mod helpers;
mod server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Hello, world!");
    let service = init_services(&"info".to_string());
    api_server(service.clone(), Server {
        host: "localhost".to_string(),
        port: "9090".to_string(),
        log_level: "info".to_string(),
    }).await
}
