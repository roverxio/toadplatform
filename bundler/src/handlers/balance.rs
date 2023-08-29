use actix_web::{HttpRequest, HttpResponse, Responder, web};
use actix_web::web::{Path};
use ethers::providers::{Http, Middleware, Provider};
use ethers::types::Address;
use serde::Serialize;

use crate::provider::helpers::get_user;

#[derive(Serialize)]
pub struct Response {
}
pub async fn balance_of(
    _web3_provider: web::Data<Provider<Http>>,
    req: HttpRequest,
    entity: Path<String>,
) -> Result<impl Responder> {
    let user = get_user(req);
    println!("User {}", user);
    println!("Entity {}", entity);
    let addr:Address = "0xb9aac8493FeD72323956DF3aC32FAa751f6fD6e4".parse().unwrap();
    match _web3_provider.get_balance(addr, None).await {

        Ok(balance) => Ok(web::Json(balance)),
        Err(..) => Ok::(web:Json()),
    };
}

#[allow(dead_code)]
pub async fn get_balance() {
    unimplemented!();
}
// /v1/admin/balance/relayer
// /v1/admin/balance/paymaster

