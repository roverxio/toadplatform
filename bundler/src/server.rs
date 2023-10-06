use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use dotenvy::dotenv;
use env_logger::{init_from_env, Env};
use ethers::middleware::SignerMiddleware;
use ethers_signers::{LocalWallet, Signer};
use log::info;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

use crate::bundler::Bundler;
use crate::contracts::entrypoint_provider::EntryPointProvider;
use crate::db::connection::DatabaseConnection;
use crate::db::dao::{TransactionDao, UserOperationDao, WalletDao};
use crate::models::config::server::Server;
use crate::provider::Web3Client;
use crate::routes::routes;
use crate::services::hello_world_service::HelloWorldService;
use crate::services::TransferService;
use crate::{CONFIG, PROVIDER};

#[derive(Clone)]
pub struct ToadService {
    pub hello_world_service: HelloWorldService,
    pub transfer_service: TransferService,
    pub web3_client: Web3Client,
    pub db_pool: Pool<Postgres>,
}

pub async fn init_services() -> ToadService {
    init_logging();
    info!("Starting server...");
    // contract providers
    let client = Arc::new(PROVIDER.clone());
    let entrypoint =
        EntryPointProvider::init_abi(CONFIG.get_chain().entrypoint_address, client.clone());

    //wallets
    let relayer_wallet: LocalWallet = std::env::var("WALLET_PRIVATE_KEY")
        .expect("WALLET_PRIVATE_KEY must be set")
        .parse::<LocalWallet>()
        .unwrap();

    //signers
    let bundler_signer = SignerMiddleware::new(
        client.clone(),
        relayer_wallet
            .clone()
            .with_chain_id(CONFIG.get_chain().chain_id),
    );

    //daos
    let pool = DatabaseConnection::init().await;
    let wallet_dao = WalletDao { pool: pool.clone() };
    let transaction_dao = TransactionDao { pool: pool.clone() };
    let user_operations_dao = UserOperationDao { pool: pool.clone() };

    // providers
    let entrypoint_provider = EntryPointProvider {
        abi: entrypoint.clone(),
    };
    let bundler = Bundler {
        signer: bundler_signer.clone(),
        entrypoint: entrypoint_provider.clone(),
    };

    // Services
    let hello_world_service = HelloWorldService {};
    let transfer_service = TransferService {
        wallet_dao: wallet_dao.clone(),
        transaction_dao: transaction_dao.clone(),
        user_operations_dao: user_operations_dao.clone(),
        entrypoint_provider: entrypoint_provider.clone(),
        bundler: bundler.clone(),
    };

    ToadService {
        hello_world_service,
        transfer_service,
        web3_client: Web3Client::new(client.clone()),
        db_pool: pool,
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
            .app_data(Data::new(service.transfer_service.clone()))
            .app_data(Data::new(service.web3_client.clone()))
            .app_data(Data::new(service.db_pool.clone()))
    })
    .bind(server.url())?
    .run()
    .await
}
