#[macro_use]
extern crate lazy_static;

use crate::models::config::server::Server;
use crate::models::config::settings;
use crate::server::{api_server, init_services};

mod handlers;
mod routes;
mod errors;
mod services;
mod models;
mod helpers;
mod server;

lazy_static! {
    static ref CONFIG: models::config::settings::Settings = settings::Settings::new().expect("Failed to load config.");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server...");
    let service = init_services(&CONFIG.log.level.clone());
    api_server(service.clone(), Server {
        host: CONFIG.server.url.clone(),
        port: CONFIG.server.port.clone().to_string(),
        log_level: CONFIG.log.level.clone(),
    }).await
}
