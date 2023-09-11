use actix_web::web::{Data, Json};
use actix_web::{Error, HttpRequest, HttpResponse};
use log::info;

use crate::errors::ApiError;
use crate::models::response::base_response::BaseResponse;
use crate::models::transfer::transfer_execute_request::TransferExecuteRequest;
use crate::models::transfer::transfer_request::TransferRequest;
use crate::models::transfer::transfer_response::TransferResponse;
use crate::provider::helpers::{get_user, respond_json};
use crate::services::transfer::transfer_service::TransferService;
use crate::services::transfer::transfer_service_v2::TransferServiceV2;

pub async fn transfer(
    service: Data<TransferService>,
    body: Json<TransferRequest>,
    req: HttpRequest,
) -> Result<Json<BaseResponse<TransferResponse>>, ApiError> {
    let body = body.into_inner();
    let data = service
        .transfer_funds(
            body.get_receiver(),
            body.get_value(),
            body.metadata.get_currency(),
            &get_user(req),
        )
        .await?;
    respond_json(data)
}

pub async fn init_transfer(
    service: Data<TransferServiceV2>,
    body: Json<TransferRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let data = service
        .init(
            body.get_receiver(),
            body.get_value(),
            body.metadata.get_currency(),
            &get_user(req),
        )
        .await;
    Ok(HttpResponse::Ok().json(BaseResponse {
        data: data.unwrap(),
        err: Default::default(),
    }))
}

pub async fn execute_transfer(
    service: Data<TransferServiceV2>,
    body: Json<TransferExecuteRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    info!("execute_transfer");
    get_user(req);
    let data = service.execute(body.transaction_id.clone(), body.signature.clone());
    Ok(HttpResponse::Ok().json(BaseResponse {
        data: data.unwrap(),
        err: Default::default(),
    }))
}
