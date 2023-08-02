use actix_web::web;
use actix_web::web::ServiceConfig;
use crate::CONFIG;

use crate::handlers::hello_world::hello_world;
use crate::handlers::wallet::get_address;

pub fn routes(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope(&CONFIG.server.prefix.clone().to_string())
            .service(
                web::scope("v1")
                    .route("hello", web::get().to(hello_world))
                    .route("address", web::get().to(get_address))
            )
    );
}
