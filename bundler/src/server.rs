use std::sync::Arc;

use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use dotenvy::dotenv;
use env_logger::{init_from_env, Env};
use ethers::middleware::SignerMiddleware;
use ethers::types::Address;
use ethers_signers::{LocalWallet, Signer};
use log::info;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::bundler::bundler::Bundler;
use crate::contracts::entrypoint_provider::EntryPointProvider;
use crate::contracts::simple_account_factory_provider::SimpleAccountFactoryProvider;
use crate::contracts::simple_account_provider::SimpleAccountProvider;
use crate::contracts::usdc_provider::USDCProvider;
use crate::db::connection::establish_connection;
use crate::db::dao::transaction_dao::TransactionDao;
use crate::db::dao::user_op_hash_dao::UserOpHashDao;
use crate::db::dao::wallet_dao::WalletDao;
use crate::models::config::server::Server;
use crate::provider::paymaster_provider::PaymasterProvider;
use crate::provider::verifying_paymaster_helper::get_verifying_paymaster_abi;
use crate::routes::routes;
use crate::services::admin_service::AdminService;
use crate::services::balance_service::BalanceService;
use crate::services::hello_world_service::HelloWorldService;
use crate::services::metada_service::MetadataService;
use crate::services::transfer_service::TransferService;
use crate::services::wallet_service::WalletService;
use crate::{CONFIG, PROVIDER};

#[derive(Clone)]
pub struct ToadService {
    pub hello_world_service: HelloWorldService,
    pub wallet_service: WalletService,
    pub balance_service: BalanceService,
    pub transfer_service: TransferService,
    pub admin_service: AdminService,
    pub metadata_service: MetadataService,
    pub db_pool: Pool<SqliteConnectionManager>,
}

pub fn init_services() -> ToadService {
    init_logging();
    info!("Starting server...");
    // contract providers
    let client = Arc::new(PROVIDER.clone());
    let simple_account_factory_provider =
        SimpleAccountFactoryProvider::init_abi(&CONFIG.run_config.current_chain, client.clone());
    let erc20_provider = USDCProvider::init_abi(&CONFIG.run_config.current_chain, client.clone());
    let entrypoint = EntryPointProvider::init_abi(&CONFIG.run_config.current_chain, client.clone());
    let simple_account_provider = SimpleAccountProvider::init_abi(client.clone(), Address::zero());
    let verifying_paymaster_provider =
        get_verifying_paymaster_abi(&CONFIG.run_config.current_chain, client.clone());
    //signers
    let verifying_paymaster_signer: LocalWallet = std::env::var("VERIFYING_PAYMASTER_PRIVATE_KEY")
        .expect("VERIFYING_PAYMASTER_PRIVATE_KEY must be set")
        .parse::<LocalWallet>()
        .unwrap();
    let wallet_signer: LocalWallet = std::env::var("WALLET_PRIVATE_KEY")
        .expect("WALLET_PRIVATE_KEY must be set")
        .parse::<LocalWallet>()
        .unwrap();
    let signing_client = SignerMiddleware::new(
        client.clone(),
        wallet_signer
            .clone()
            .with_chain_id(CONFIG.chains[&CONFIG.run_config.current_chain].chain_id),
    );
    let bundler_signing_client = SignerMiddleware::new(
        client.clone(),
        wallet_signer
            .clone()
            .with_chain_id(CONFIG.chains[&CONFIG.run_config.current_chain].chain_id),
    );

    //daos
    let pool = establish_connection(CONFIG.database.file.clone());
    let wallet_dao = WalletDao { pool: pool.clone() };
    let transaction_dao = TransactionDao { pool: pool.clone() };
    let user_op_hash_dao = UserOpHashDao { pool: pool.clone() };

    // providers
    let verify_paymaster_provider = PaymasterProvider {
        provider: verifying_paymaster_provider.clone(),
    };
    let entrypoint_provider = EntryPointProvider {
        abi: entrypoint.clone(),
    };
    let bundler = Bundler {
        signing_client: bundler_signing_client.clone(),
        entrypoint: entrypoint_provider.clone(),
    };

    // Services
    let hello_world_service = HelloWorldService {};
    let wallet_service = WalletService {
        wallet_dao: wallet_dao.clone(),
        simple_account_factory_provider: simple_account_factory_provider.clone(),
        client: client.clone(),
    };
    let balance_service = BalanceService {
        wallet_dao: wallet_dao.clone(),
        erc20_provider: erc20_provider.clone(),
    };
    let transfer_service = TransferService {
        wallet_dao: wallet_dao.clone(),
        transaction_dao: transaction_dao.clone(),
        user_op_hash_dao: user_op_hash_dao.clone(),
        usdc_provider: erc20_provider.clone(),
        entrypoint_provider: entrypoint_provider.clone(),
        simple_account_provider: simple_account_provider.clone(),
        simple_account_factory_provider: simple_account_factory_provider.clone(),
        verifying_paymaster_provider: verifying_paymaster_provider.clone(),
        verifying_paymaster_signer: verifying_paymaster_signer.clone(),
        wallet_singer: wallet_signer.clone(),
        bundler: bundler.clone(),
    };
    let admin_service = AdminService {
        paymaster_provider: verify_paymaster_provider.clone(),
        entrypoint_provider: entrypoint_provider.clone(),
        signing_client: signing_client.clone(),
    };
    let metadata_service = MetadataService {};

    ToadService {
        hello_world_service,
        wallet_service,
        balance_service,
        transfer_service,
        admin_service,
        metadata_service,
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
            .app_data(Data::new(service.wallet_service.clone()))
            .app_data(Data::new(service.balance_service.clone()))
            .app_data(Data::new(service.transfer_service.clone()))
            .app_data(Data::new(service.admin_service.clone()))
            .app_data(Data::new(service.metadata_service.clone()))
            .app_data(service.db_pool.clone())
    })
    .bind(server.url())?
    .run()
    .await
}
