use actix_web::web::{Data, Json, Path, Query};
use actix_web::{HttpRequest, HttpResponse};
use sqlx::{Pool, Postgres};

use crate::errors::AdminError;
use crate::models::admin::AddMetadataRequest;
use crate::models::admin::PaymasterTopup;
use crate::models::response::base_response::BaseResponse;
use crate::models::wallet::balance_request::BalanceRequest;
use crate::provider::helpers::get_user;
use crate::provider::web3_client::Web3Client;
use crate::services::AdminService;
use crate::CONFIG;

pub async fn topup_paymaster_deposit(
    provider: Data<Web3Client>,
    body: Json<PaymasterTopup>,
    req: HttpRequest,
    paymaster: Path<String>,
) -> Result<HttpResponse, AdminError> {
    if is_not_admin(get_user(req)) {
        return Err(AdminError::Unauthorized);
    }
    let paymaster_req = body.into_inner();
    let response = AdminService::topup_paymaster_deposit(
        provider.as_ref(),
        paymaster_req.value,
        paymaster.clone(),
        paymaster_req.metadata,
    )
    .await?;
    Ok(HttpResponse::Ok().json(BaseResponse::init(response)))
}

pub async fn admin_get_balance(
    provider: Data<Web3Client>,
    body: Query<BalanceRequest>,
    req: HttpRequest,
    entity: Path<String>,
) -> Result<HttpResponse, AdminError> {
    if is_not_admin(get_user(req)) {
        return Err(AdminError::Unauthorized);
    }
    let response = AdminService::get_balance(
        provider.get_ref(),
        entity.clone(),
        body.get_balance_request(),
    )
    .await?;
    Ok(HttpResponse::Ok().json(BaseResponse::init(response)))
}

pub async fn add_currency_metadata(
    pool: Data<Pool<Postgres>>,
    body: Json<AddMetadataRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, AdminError> {
    if is_not_admin(get_user(req)) {
        return Err(AdminError::Unauthorized);
    }
    let response = AdminService::add_currency_metadata(pool.get_ref(), body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(BaseResponse::init(response)))
}

fn is_not_admin(user: String) -> bool {
    !CONFIG.get_admins().contains(&user)
}
