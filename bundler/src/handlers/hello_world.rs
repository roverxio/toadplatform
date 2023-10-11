use actix_web::{Error, HttpResponse};

use crate::models::response::BaseResponse;
use crate::services::HelloWorldService;

pub async fn hello_world() -> Result<HttpResponse, Error> {
    let hello = HelloWorldService::hello_world()?;
    Ok(HttpResponse::Ok().json(BaseResponse::init(hello)))
}

#[cfg(test)]
mod test {
    use actix_web::{test, web, App};
    use reqwest::StatusCode;

    use crate::handlers::hello_world::hello_world;

    #[actix_web::test]
    async fn test_hello_world() {
        let app = test::init_service(
            App::new().service(web::scope("v1").route("hello", web::get().to(hello_world))),
        )
        .await;
        let req = test::TestRequest::get().uri("/v1/hello").to_request();
        let result = test::call_service(&app, req).await;
        assert_eq!(result.status(), StatusCode::OK);
    }
}
