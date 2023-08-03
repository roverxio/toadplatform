use actix_web::{App, HttpServer};
use actix_web::middleware::Logger;
use actix_web::web::Data;
use dotenvy::dotenv;
use env_logger::{Env, init_from_env};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use crate::CONFIG;

use crate::models::config::server::Server;
use crate::routes::routes;
use crate::services::admin_service::AdminService;
use crate::services::balance_service::BalanceService;
use crate::services::hello_world_service::HelloWorldService;
use crate::services::transfer_service::TransactionService;
use crate::services::wallet_service::WalletService;

#[derive(Clone)]
pub struct ToadService {
    pub hello_world_service: HelloWorldService,
    pub wallet_service: WalletService,
    pub balance_service: BalanceService,
    pub transfer_service: TransactionService,
    pub admin_service: AdminService,
}

pub fn init_services(
    pool: Pool<SqliteConnectionManager>
) -> ToadService {
    println!("pool: {:?}", pool);
    init_logging();
    // Services
    let hello_world_service = HelloWorldService {};
    let wallet_service = WalletService {};
    let balance_service = BalanceService {};
    let transfer_service = TransactionService {};
    let admin_service = AdminService {};

    ToadService {
        hello_world_service,
        wallet_service,
        balance_service,
        transfer_service,
        admin_service,
    }
}

fn init_logging() {
    let log_level = CONFIG.log.level.as_str();
    std::env::set_var("RUST_LOG", log_level);
    init_from_env(Env::default().default_filter_or(log_level));
}

pub async fn api_server(service: ToadService, server: Server) -> std::io::Result<()> {
    dotenv().ok();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .configure(routes)
            .app_data(Data::new(service.hello_world_service.clone()))
            .app_data(Data::new(service.wallet_service.clone()))
            .app_data(Data::new(service.balance_service.clone()))
            .app_data(Data::new(service.transfer_service.clone()))
            .app_data(Data::new(service.admin_service.clone()))
    })
        .bind(server.url())?
        .run()
        .await
}
