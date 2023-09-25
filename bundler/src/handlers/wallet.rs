use actix_web::web::{Data, Json, Query, ReqData};
use actix_web::{Error, HttpRequest, HttpResponse};
use sqlx::{Pool, Postgres};

use crate::db::dao::wallet_dao::User;
use crate::errors::balance::BalanceError;
use crate::errors::errors::ApiError;
use crate::models::response::base_response::BaseResponse;
use crate::models::transaction::list_transactions_params::ListTransactionsParams;
use crate::models::transaction::poll_transaction_params::PollTransactionParams;
use crate::models::transaction::transaction::Transaction;
use crate::models::wallet::address_response::AddressResponse;
use crate::models::wallet::balance_request::BalanceRequest;
use crate::provider::helpers::{get_user_wallet, respond_json};
use crate::provider::web3_client::Web3Client;
use crate::services::balance_service::BalanceService;
use crate::services::transfer_service::TransferService;
use crate::services::wallet_service::WalletService;

pub async fn get_address(
    service: Data<WalletService>,
    user: ReqData<User>,
    req: HttpRequest,
) -> Result<Json<BaseResponse<AddressResponse>>, ApiError> {
    let wallet_address = service
        .get_wallet_address(user.into_inner(), get_user_wallet(req))
        .await?;
    respond_json(wallet_address)
}

pub async fn get_balance(
    pool: Data<Pool<Postgres>>,
    provider: Data<Web3Client>,
    body: Query<BalanceRequest>,
    user: ReqData<User>,
) -> Result<HttpResponse, BalanceError> {
    let balance_request = body.get_balance_request();
    let data = BalanceService::get_wallet_balance(
        pool.get_ref(),
        provider.get_ref(),
        &balance_request.get_chain(),
        &balance_request.get_currency(),
        user.into_inner(),
    )
    .await?;
    Ok(HttpResponse::Ok().json(BaseResponse::init(data)))
}

pub async fn list_transactions(
    service: Data<WalletService>,
    query: Query<ListTransactionsParams>,
    user: ReqData<User>,
) -> Result<Json<BaseResponse<Vec<Transaction>>>, ApiError> {
    let query_params = query.into_inner();
    let data = service
        .list_transactions(
            query_params.page_size.unwrap_or(10),
            query_params.id,
            user.into_inner(),
        )
        .await;
    respond_json(data)
}

pub async fn poll_transaction(
    db_pool: Data<Pool<Postgres>>,
    query: Query<PollTransactionParams>,
    user: ReqData<User>,
) -> Result<HttpResponse, Error> {
    let transaction = TransferService::get_status(
        db_pool.get_ref(),
        query.transaction_id.clone(),
        user.into_inner(),
    )
    .await
    .unwrap();

    Ok(HttpResponse::Ok().json(BaseResponse {
        data: transaction,
        err: Default::default(),
    }))
}

// Chain -> localhost
#[cfg(test)]
mod tests {
    use crate::db::connection::DatabaseConnection;
    use crate::provider::web3_client::Web3Client;
    use crate::routes::routes;
    use crate::PROVIDER;
    use actix_web::http::StatusCode;
    use actix_web::web::Data;
    use actix_web::{test, App};
    use reqwest::header;
    use std::sync::Arc;

    #[actix_web::test]
    async fn test_get_usdc_balance() {
        let pool = DatabaseConnection::init().await;
        let web3_client = Web3Client::new(Arc::new(PROVIDER.clone()));
        let app = App::new()
            .configure(routes)
            .app_data(Data::new(web3_client.clone()))
            .app_data(Data::new(pool.clone()));

        let mut app = test::init_service(app).await;

        let req = test::TestRequest::get()
            .uri("/app/v1/user/balance?q=eyAgICAiY2hhaW4iOiAibG9jYWxob3N0IiwgICAgImN1cnJlbmN5IjogIlVTREMifQ==")
            .append_header((header::AUTHORIZATION, "Bearer eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IlRZT2dnXy01RU9FYmxhWS1WVlJZcVZhREFncHRuZktWNDUzNU1aUEMwdzAifQ.eyJpYXQiOjE2OTU2MzkwMTYsImF1ZCI6IkJKRDIxdWhIeHZWS0laa2JqT1lua1NlUkdCd2JvbDJwT212aHFWN2ZnNWR6NDJaRGhiTHdOemt0cDA0YU93VENsZnRVU094eHJqZVQyTUFlQUFpb0drMCIsIm5vbmNlIjoiMDI4MTdmZTVlNjBlODkyYTk0ZGQ5NTQyMTNhMWM4YjkxOGU4MjE2NDRkNzBhZjY4Y2FlM2JhYmU1YTdkNmMyZjQwIiwiaXNzIjoiaHR0cHM6Ly9hcGktYXV0aC53ZWIzYXV0aC5pbyIsIndhbGxldHMiOlt7InB1YmxpY19rZXkiOiIwM2RjYmI2ZGRhNTE5ZjZiNTg0NGExMmRlNDY4YTU3MmYwNTdlN2VlNmNkYWI2M2M2MGIxNDA1NjkzZWYxYTMxODciLCJ0eXBlIjoid2ViM2F1dGhfYXBwX2tleSIsImN1cnZlIjoic2VjcDI1NmsxIn1dLCJlbWFpbCI6InZpc2hudWt2bnZ2QGdtYWlsLmNvbSIsIm5hbWUiOiJWaXNobnUgVmFyZGhhbiIsInByb2ZpbGVJbWFnZSI6Imh0dHBzOi8vbGgzLmdvb2dsZXVzZXJjb250ZW50LmNvbS9hL0FBY0hUdGREWlVfbUFNWjk3bUFiaWZWaWVLSEIwbndqczdyQkl2a3VzM3kxVDUxZWJ3PXM5Ni1jIiwidmVyaWZpZXIiOiJ0b2FkLWNhc2gtdmVyaWZpZXIiLCJ2ZXJpZmllcklkIjoiSkNIRUlkNzUxWGczTnMwQkp3d0N3em9zQUtqMSIsImFnZ3JlZ2F0ZVZlcmlmaWVyIjoidG9hZC1jYXNoLXZlcmlmaWVyIiwiZXhwIjoxNjk1NzI1NDE2fQ.-d22vJ2p1z5BRKYLSa29L5b0ygMp0IvdKguPcWlBWbbKSqTKo_PFKXal8CK7lKZT9vIOMYd15kgrT5bHsYMM5A"))
            .to_request();

        let result = test::call_service(&mut app, req).await;
        assert_eq!(result.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_get_native_balance() {
        let pool = DatabaseConnection::init().await;
        let web3_client = Web3Client::new(Arc::new(PROVIDER.clone()));
        let app = App::new()
            .configure(routes)
            .app_data(Data::new(web3_client.clone()))
            .app_data(Data::new(pool.clone()));

        let mut app = test::init_service(app).await;

        let req = test::TestRequest::get()
            .uri("/app/v1/user/balance?q=eyAgICAiY2hhaW4iOiAibG9jYWxob3N0IiwgICAgImN1cnJlbmN5IjogIkVUSCJ9")
            .append_header((header::AUTHORIZATION, "Bearer eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IlRZT2dnXy01RU9FYmxhWS1WVlJZcVZhREFncHRuZktWNDUzNU1aUEMwdzAifQ.eyJpYXQiOjE2OTU2MzkwMTYsImF1ZCI6IkJKRDIxdWhIeHZWS0laa2JqT1lua1NlUkdCd2JvbDJwT212aHFWN2ZnNWR6NDJaRGhiTHdOemt0cDA0YU93VENsZnRVU094eHJqZVQyTUFlQUFpb0drMCIsIm5vbmNlIjoiMDI4MTdmZTVlNjBlODkyYTk0ZGQ5NTQyMTNhMWM4YjkxOGU4MjE2NDRkNzBhZjY4Y2FlM2JhYmU1YTdkNmMyZjQwIiwiaXNzIjoiaHR0cHM6Ly9hcGktYXV0aC53ZWIzYXV0aC5pbyIsIndhbGxldHMiOlt7InB1YmxpY19rZXkiOiIwM2RjYmI2ZGRhNTE5ZjZiNTg0NGExMmRlNDY4YTU3MmYwNTdlN2VlNmNkYWI2M2M2MGIxNDA1NjkzZWYxYTMxODciLCJ0eXBlIjoid2ViM2F1dGhfYXBwX2tleSIsImN1cnZlIjoic2VjcDI1NmsxIn1dLCJlbWFpbCI6InZpc2hudWt2bnZ2QGdtYWlsLmNvbSIsIm5hbWUiOiJWaXNobnUgVmFyZGhhbiIsInByb2ZpbGVJbWFnZSI6Imh0dHBzOi8vbGgzLmdvb2dsZXVzZXJjb250ZW50LmNvbS9hL0FBY0hUdGREWlVfbUFNWjk3bUFiaWZWaWVLSEIwbndqczdyQkl2a3VzM3kxVDUxZWJ3PXM5Ni1jIiwidmVyaWZpZXIiOiJ0b2FkLWNhc2gtdmVyaWZpZXIiLCJ2ZXJpZmllcklkIjoiSkNIRUlkNzUxWGczTnMwQkp3d0N3em9zQUtqMSIsImFnZ3JlZ2F0ZVZlcmlmaWVyIjoidG9hZC1jYXNoLXZlcmlmaWVyIiwiZXhwIjoxNjk1NzI1NDE2fQ.-d22vJ2p1z5BRKYLSa29L5b0ygMp0IvdKguPcWlBWbbKSqTKo_PFKXal8CK7lKZT9vIOMYd15kgrT5bHsYMM5A"))
            .to_request();

        let result = test::call_service(&mut app, req).await;
        assert_eq!(result.status(), StatusCode::OK);
    }
}
