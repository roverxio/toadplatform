use actix_web::web::Data;
use actix_web::HttpResponse;
use sqlx::{Pool, Postgres};

use crate::errors::errors::ApiError;
use crate::errors::MetadataError;
use crate::models::response::base_response::BaseResponse;
use crate::services::token_metadata_service::TokenMetadataService;

pub async fn get_metadata(pool: Data<Pool<Postgres>>) -> Result<HttpResponse, MetadataError> {
    let response = TokenMetadataService::get_chain(pool.get_ref()).await?;
    Ok(HttpResponse::Ok().json(BaseResponse::init(response)))
}

pub async fn get_metadata_v2(pool: Data<Pool<Postgres>>) -> Result<HttpResponse, ApiError> {
    let response = TokenMetadataService::get_chain_v2(pool.get_ref()).await?;
    Ok(HttpResponse::Ok().json(BaseResponse::init(response)))
}
