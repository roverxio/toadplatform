use crate::db::dao::wallet_dao::User;
use actix_web::web::{Data, Json, ReqData};
use actix_web::{Error, HttpResponse};

use crate::models::response::base_response::BaseResponse;
use crate::models::transfer::transfer_execute_request::TransferExecuteRequest;
use crate::models::transfer::transfer_request::TransferRequest;
use crate::services::transfer_service::TransferService;

pub async fn init_transfer(
    service: Data<TransferService>,
    body: Json<TransferRequest>,
    user: ReqData<User>,
) -> Result<HttpResponse, Error> {
    let data = service
        .init(
            body.get_receiver(),
            body.get_value(),
            body.metadata.get_currency(),
            user.into_inner(),
        )
        .await;
    Ok(HttpResponse::Ok().json(BaseResponse {
        data: data.unwrap(),
        err: Default::default(),
    }))
}

pub async fn execute_transfer(
    service: Data<TransferService>,
    body: Json<TransferExecuteRequest>,
    req: ReqData<User>,
) -> Result<HttpResponse, Error> {
    let data = service
        .execute(
            body.transaction_id.clone(),
            body.get_signature(),
            req.into_inner(),
        )
        .await;
    Ok(HttpResponse::Ok().json(BaseResponse {
        data: data.unwrap(),
        err: Default::default(),
    }))
}
