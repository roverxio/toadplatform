use actix_web::web;
use actix_web::web::ServiceConfig;

use crate::handlers::hello_world::hello_world;

pub fn routes(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("app")
            .service(
                web::scope("v1")
                    .route("hello", web::get().to(hello_world))
            )
    );
}
