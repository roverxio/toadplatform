use actix_web::web::{Data, Json, Path, Query};
use actix_web::HttpRequest;

use crate::errors::ApiError;
use crate::models::admin::add_metadata_request::AddMetadataRequest;
use crate::models::admin::metadata_response::MetadataResponse;
use crate::models::admin::paymaster_topup::PaymasterTopup;
use crate::models::response::base_response::BaseResponse;
use crate::models::transfer::transfer_response::TransferResponse;
use crate::models::wallet::balance_request::BalanceRequest;
use crate::models::wallet::balance_response::BalanceResponse;
use crate::provider::helpers::{get_user, respond_json};
use crate::services::admin_service::AdminService;

pub async fn topup_paymaster_deposit(
    service: Data<AdminService>,
    body: Json<PaymasterTopup>,
    req: HttpRequest,
    paymaster: Path<String>,
) -> Result<Json<BaseResponse<TransferResponse>>, ApiError> {
    if is_not_admin(get_user(req)) {
        return Err(ApiError::BadRequest("Invalid credentials".to_string()));
    }
    let req = body.into_inner();
    let response = service
        .topup_paymaster_deposit(req.value, paymaster.clone(), req.metadata)
        .await?;
    respond_json(response)
}

pub async fn admin_get_balance(
    service: Data<AdminService>,
    body: Query<BalanceRequest>,
    req: HttpRequest,
    entity: Path<String>,
) -> Result<Json<BaseResponse<BalanceResponse>>, ApiError> {
    if is_not_admin(get_user(req)) {
        return Err(ApiError::BadRequest("Invalid credentials".to_string()));
    }
    let response = service
        .get_balance(entity.clone(), body.get_balance_request())
        .await?;
    respond_json(response)
}

pub async fn add_currency_metadata(
    service: Data<AdminService>,
    body: Json<AddMetadataRequest>,
    req: HttpRequest,
) -> Result<Json<BaseResponse<MetadataResponse>>, ApiError> {
    if is_not_admin(get_user(req)) {
        return Err(ApiError::BadRequest("Invalid credentials".to_string()));
    }
    let response = service.add_currency_metadata(body.into_inner()).await?;
    respond_json(response)
}

fn is_not_admin(user: String) -> bool {
    let admins_env = std::env::var("ADMIN").expect("ADMIN env variable not set");
    let admins = admins_env.split(",").collect::<Vec<&str>>();
    !admins.contains(&user.as_str())
}
