use actix_web::web::{Data, Json};
use actix_web::{HttpRequest, HttpResponse};
use log::info;

use crate::errors::ApiError;
use crate::models::response::base_response::BaseResponse;
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
) -> Result<HttpResponse, ApiError> {
    info!("init_transfer");
    let data = service
        .transfer_init(
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

pub async fn execute_transfer() -> Result<HttpResponse, ApiError> {
    info!("execute_transfer");
    Ok(HttpResponse::Ok().finish())
}
