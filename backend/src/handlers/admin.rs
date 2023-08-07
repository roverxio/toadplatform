use actix_web::web::{Data, Json};
use actix_web::HttpRequest;

use crate::errors::ApiError;
use crate::models::admin::paymaster_topup::PaymasterTopup;
use crate::models::response::base_response::BaseResponse;
use crate::models::transfer::transfer_response::TransactionResponse;
use crate::provider::helpers::{get_user, respond_json};
use crate::services::admin_service::AdminService;

pub async fn topup_paymaster_deposit(
    service: Data<AdminService>,
    body: Json<PaymasterTopup>,
    req: HttpRequest,
) -> Result<Json<BaseResponse<TransactionResponse>>, ApiError> {
    println!("user -> {}", get_user(req));
    let response = service.topup_paymaster_deposit(body.into_inner())?;
    respond_json(response)
}
