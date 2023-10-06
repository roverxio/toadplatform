use actix_web::web::{Data, Json, ReqData};
use actix_web::{Error, HttpResponse};
use sqlx::{Pool, Postgres};

use crate::db::dao::User;
use crate::models::response::base_response::BaseResponse;
use crate::models::transfer::transfer_execute_request::TransferExecuteRequest;
use crate::models::transfer::transfer_request::TransferRequest;
use crate::provider::Web3Client;
use crate::services::TransferService;

pub async fn init_transfer(
    pool: Data<Pool<Postgres>>,
    provider: Data<Web3Client>,
    body: Json<TransferRequest>,
    user: ReqData<User>,
) -> Result<HttpResponse, Error> {
    let data = TransferService::init(
        pool.get_ref(),
        provider.get_ref(),
        body.get_receiver(),
        body.get_value(),
        body.metadata.get_currency(),
        user.into_inner(),
    )
    .await?;
    Ok(HttpResponse::Ok().json(BaseResponse::init(data)))
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
