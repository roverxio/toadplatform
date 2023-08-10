use actix_web::web::{Data, Json, Path};
use actix_web::HttpRequest;

use crate::errors::ApiError;
use crate::models::admin::paymaster_topup::PaymasterTopup;
use crate::models::response::base_response::BaseResponse;
use crate::models::transfer::transfer_response::TransactionResponse;
use crate::models::wallet::balance_response::BalanceResponse;
use crate::provider::helpers::{get_user, respond_json};
use crate::services::admin_service::AdminService;

pub async fn topup_paymaster_deposit(
    service: Data<AdminService>,
    body: Json<PaymasterTopup>,
    req: HttpRequest,
    paymaster: Path<String>,
) -> Result<Json<BaseResponse<TransactionResponse>>, ApiError> {
    println!("user -> {}", get_user(req));
    println!("paymaster -> {}", paymaster);
    let response = service.topup_paymaster_deposit(body.into_inner())?;
    respond_json(response)
}

pub async fn admin_get_balance(
    service: Data<AdminService>,
    req: HttpRequest,
    entity: Path<String>,
) -> Result<Json<BaseResponse<BalanceResponse>>, ApiError> {
    println!("user -> {}", get_user(req));
    let response = service.get_balance(entity.clone())?;
    respond_json(response)
}
