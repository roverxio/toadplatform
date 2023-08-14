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

use crate::db::connection::establish_connection;
use crate::db::dao::transaction_dao::TransactionDao;
use crate::db::dao::wallet_dao::WalletDao;
use crate::models::config::server::Server;
use crate::provider::entrypoint_helper::get_entrypoint_abi;
use crate::provider::http_client::HttpClient;
use crate::provider::verifying_paymaster_helper::get_verifying_paymaster_abi;
use crate::provider::web3_provider::Web3Provider;
use crate::routes::routes;
use crate::services::admin_service::AdminService;
use crate::services::balance_service::BalanceService;
use crate::services::hello_world_service::HelloWorldService;
use crate::services::metada_service::MetadataService;
use crate::services::transfer_service::TransactionService;
use crate::services::wallet_service::WalletService;
use crate::{CONFIG, PROVIDER};
use crate::provider::paymaster_provider::PaymasterProvider;

#[derive(Clone)]
pub struct ToadService {
    pub hello_world_service: HelloWorldService,
    pub wallet_service: WalletService,
    pub balance_service: BalanceService,
    pub transfer_service: TransactionService,
    pub admin_service: AdminService,
    pub metadata_service: MetadataService,
}

pub fn init_services() -> ToadService {
    init_logging();
    info!("Starting server...");
    // contract providers
    let client = Arc::new(PROVIDER.clone());
    let simple_account_factory_provider = Web3Provider::get_simple_account_factory_abi(
        &CONFIG.run_config.current_chain,
        client.clone(),
    );
    let erc20_provider =
        Web3Provider::get_erc20_abi(&CONFIG.run_config.current_chain, client.clone());
    let entrypoint_provider = get_entrypoint_abi(&CONFIG.run_config.current_chain, client.clone());
    let simple_account_provider = Web3Provider::get_simpleaccount_abi(client.clone(), Address::zero());
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
    // http client
    let http_client = HttpClient {
        client: reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap(),
    };

    //daos
    let pool = establish_connection(CONFIG.database.file.clone());
    let wallet_dao = WalletDao { pool: pool.clone() };
    let transaction_dao = TransactionDao { pool: pool.clone() };

    // providers
    let verify_paymaster_provider = PaymasterProvider {
        provider: verifying_paymaster_provider.clone(),
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
    let transfer_service = TransactionService {
        wallet_dao: wallet_dao.clone(),
        transaction_dao: transaction_dao.clone(),
        usdc_provider: erc20_provider.clone(),
        entrypoint_provider: entrypoint_provider.clone(),
        simple_account_provider: simple_account_provider.clone(),
        simple_account_factory_provider: simple_account_factory_provider.clone(),
        verifying_paymaster_provider: verifying_paymaster_provider.clone(),
        verifying_paymaster_signer: verifying_paymaster_signer.clone(),
        wallet_singer: wallet_signer.clone(),
        signing_client: signing_client.clone(),
        http_client: http_client.clone(),
    };
    let admin_service = AdminService {
        paymaster_provider: verify_paymaster_provider.clone(),
    };
    let metadata_service = MetadataService {};

    ToadService {
        hello_world_service,
        wallet_service,
        balance_service,
        transfer_service,
        admin_service,
        metadata_service,
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
            .app_data(Data::new(service.metadata_service.clone()))
    })
    .bind(server.url())?
    .run()
    .await
}
