use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use dotenvy::dotenv;
use env_logger::{init_from_env, Env};
use ethers::providers::{Http, Provider};
use log::info;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

use crate::db::connection::DatabaseConnection;
use crate::models::config::server::Server;
use crate::routes::routes;
use crate::services::hello_world_service::HelloWorldService;
use crate::{CONFIG, PROVIDER};

#[derive(Clone)]
pub struct ToadService {
    pub hello_world_service: HelloWorldService,
    pub web3_client: Arc<Provider<Http>>,
    pub db_pool: Pool<Postgres>,
}

pub async fn init_services() -> ToadService {
    init_logging();
    info!("Starting server...");

    ToadService {
        hello_world_service: HelloWorldService {},
        web3_client: Arc::new(PROVIDER.clone()),
        db_pool: DatabaseConnection::init().await,
    }
}

fn init_logging() {
    let log_level = CONFIG.log.level.as_str();
    std::env::set_var("RUST_LOG", log_level);
    init_from_env(Env::default().default_filter_or(log_level));
}

pub async fn run(service: ToadService, server: Server) -> std::io::Result<()> {
    dotenv().ok();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .configure(routes)
            .app_data(Data::new(service.hello_world_service.clone()))
            .app_data(Data::new(service.web3_client.clone()))
            .app_data(Data::new(service.db_pool.clone()))
    })
    .bind(server.url())?
    .run()
    .await
}
