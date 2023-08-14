use ethers::providers::{Http, Provider};
use lazy_static::lazy_static;

use crate::models::config::server::Server;
use crate::models::config::settings::Settings;
use crate::provider::web3_provider::Web3Provider;
use crate::server::{api_server, init_services};

mod db;
mod errors;
mod handlers;
mod models;
mod provider;
mod routes;
mod server;
mod services;
mod constants;

lazy_static! {
    static ref CONFIG: Settings = Settings::new().expect("Failed to load config.");
    static ref PROVIDER: Provider<Http> =
        Web3Provider::new(CONFIG.chains[&CONFIG.run_config.current_chain].get_url());
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let service = init_services();
    api_server(
        service.clone(),
        Server {
            host: CONFIG.server.host.clone(),
            port: CONFIG.server.port.clone().to_string(),
            log_level: CONFIG.log.level.clone(),
        },
    )
    .await
}
