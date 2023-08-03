use lazy_static::lazy_static;
use crate::models::config::server::Server;
use crate::models::config::settings::Settings;
use crate::server::{api_server, init_services};

mod handlers;
mod routes;
mod errors;
mod services;
mod models;
mod server;
mod provider;

lazy_static! {
    static ref CONFIG: Settings = Settings::new().expect("Failed to load config.");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server...");
    let service = init_services(&CONFIG.log.level.clone());
    api_server(service.clone(), Server {
        host: CONFIG.server.host.clone(),
        port: CONFIG.server.port.clone().to_string(),
        log_level: CONFIG.log.level.clone(),
    }).await
}
