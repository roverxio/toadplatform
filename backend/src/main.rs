use ethers::{
    providers::{Http, Provider},
};
use ethers_signers::LocalWallet;
use lazy_static::lazy_static;
use crate::db::connection::establish_connection;

use crate::models::config::server::Server;
use crate::models::config::settings::Settings;
use crate::provider::web3_provider::Web3Provider;
use crate::server::{api_server, init_services};

mod handlers;
mod routes;
mod errors;
mod services;
mod models;
mod server;
mod provider;
mod db;

lazy_static! {
    static ref CONFIG: Settings = Settings::new().expect("Failed to load config.");
    static ref PROVIDER: Provider<Http> = Web3Provider::new(CONFIG.chains[&CONFIG.current_chain].get_url());
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server...");
    let service = init_services(establish_connection(CONFIG.database.file.clone()));
    api_server(service.clone(), Server {
        host: CONFIG.server.host.clone(),
        port: CONFIG.server.port.clone().to_string(),
        log_level: CONFIG.log.level.clone(),
    }).await
}
