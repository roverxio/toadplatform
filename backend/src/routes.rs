use actix_web::web;
use actix_web::web::ServiceConfig;

use crate::CONFIG;
use crate::handlers::admin::{admin_get_balance, topup_paymaster_deposit};
use crate::handlers::hello_world::hello_world;
use crate::handlers::metada::get_metadata;
use crate::handlers::wallet::{get_address, get_balance, transact};

pub fn routes(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope(&CONFIG.server.prefix.clone().to_string())
            .service(
                web::scope("v1")
                    .route("hello", web::get().to(hello_world))
                    .route("user/address", web::get().to(get_address))
                    .route("user/balance", web::get().to(get_balance))
                    .route("user/transact", web::post().to(transact))
                    .route("admin/deposit/{paymaster}", web::post().to(topup_paymaster_deposit))// the paymaster name
                    .route("admin/balance/{entity}", web::get().to(admin_get_balance)) // entity can be a paymaster or the EOA
                    .route("metadata", web::get().to(get_metadata))
            )
    );
}
