use actix_web::web::{Data, Json, ReqData};
use actix_web::HttpResponse;
use ethers::prelude::{Http, Provider};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

use crate::db::dao::User;
use crate::errors::TransferError;
use crate::models::response::BaseResponse;
use crate::models::transfer::{TransferExecuteRequest, TransferRequest};
use crate::services::TransferService;

pub async fn init_transfer(
    pool: Data<Pool<Postgres>>,
    provider: Data<Arc<Provider<Http>>>,
    body: Json<TransferRequest>,
    user: ReqData<User>,
) -> Result<HttpResponse, TransferError> {
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
    pool: Data<Pool<Postgres>>,
    provider: Data<Arc<Provider<Http>>>,
    body: Json<TransferExecuteRequest>,
    req: ReqData<User>,
) -> Result<HttpResponse, TransferError> {
    let data = TransferService::execute(
        pool.get_ref(),
        provider.get_ref(),
        body.transaction_id.clone(),
        body.get_signature(),
        req.into_inner(),
    )
    .await?;
    Ok(HttpResponse::Ok().json(BaseResponse::init(data)))
}
