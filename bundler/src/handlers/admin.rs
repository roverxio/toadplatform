use actix_web::web::{Data, Json, Path, Query};
use actix_web::HttpRequest;

use crate::constants::Constants;
use crate::errors::ApiError;
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
    if Constants::ADMIN != get_user(req) {
        return Err(ApiError::BadRequest("Invalid credentials".to_string()));
    }
    let response = service
        .topup_paymaster_deposit(body.into_inner(), paymaster.clone())
        .await?;
    respond_json(response)
}

pub async fn admin_get_balance(
    service: Data<AdminService>,
    body: Query<BalanceRequest>,
    req: HttpRequest,
    entity: Path<String>,
) -> Result<Json<BaseResponse<BalanceResponse>>, ApiError> {
    if Constants::ADMIN != get_user(req) {
        return Err(ApiError::BadRequest("Invalid credentials".to_string()));
    }
    let response = service
        .get_balance(entity.clone(), body.get_balance_request())
        .await?;
    respond_json(response)
}
