use actix_web::{HttpRequest, HttpResponse, web};
use actix_web::web::{Path};
use ethers::providers::{Http, Middleware, Provider};
use ethers::types::Address;
use crate::provider::helpers::get_user;


// #[derive(thiserror::Error)]
// pub enum BalanceError {
// }


// impl std::fmt::Debug for BalanceError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         todo!()
//     }
//
// }

// impl Display for BalanceError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         todo!()
//     }
// }

// impl ResponseError for BalanceError {
//     fn status_code(&self) -> StatusCode {
//         todo!()
//     }
// }


pub async fn balance_of(
    _web3_provider: web::Data<Provider<Http>>,
    req: HttpRequest,
    entity: Path<String>,
) -> HttpResponse {
    // needs access to paymaster/relayer address
    // require the name of the entity of which we need the balance
    // fetch balance in a certain currency (?)

    // let addr:Address = "0xb9aac8493FeD72323956DF3aC32FAa751f6fD6e4".parse().unwrap();
    // let balance = web3_provider.get_balance(addr, None)
    //     .await
    // return balance;

    let user = get_user(req);
    let addr:Address = "0xb9aac8493FeD72323956DF3aC32FAa751f6fD6e4".parse().unwrap();
    match _web3_provider.get_balance(addr, None) {
        Ok(balance) => println!("The balance is {}", balance),
        Err(e) => println!("Error"),
    };
    println!( "The user is {}", user);
    println!("The entity is {}", entity);

    return  HttpResponse::Ok().finish();
}

#[allow(dead_code)]
pub async fn get_balance() {
    unimplemented!();
}
// /v1/admin/balance/relayer
// /v1/admin/balance/paymaster

