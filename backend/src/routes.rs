use crate::handlers::admin::topup_paymaster_deposit;
use crate::CONFIG;
use actix_web::web;
use actix_web::web::ServiceConfig;

use crate::handlers::hello_world::hello_world;
use crate::handlers::wallet::{get_address, get_balance, transact};

pub fn routes(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope(&CONFIG.server.prefix.clone().to_string()).service(
            web::scope("v1")
                .route("hello", web::get().to(hello_world))
                .route("address", web::get().to(get_address))
                .route("balance/{entity}", web::get().to(get_balance))
                .route("transact", web::post().to(transact))
                .route("deposit/add", web::post().to(topup_paymaster_deposit)),
        ),
    );
}
