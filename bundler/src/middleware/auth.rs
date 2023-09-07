use actix_web::web::Data;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::{Either, LocalBoxFuture};
use log::error;
use sqlx::{Pool, Postgres};
use std::future::{ready, Future, Ready};
use std::pin::Pin;

use crate::db::dao::wallet_dao::WalletDao;
use crate::errors::ApiError;
use crate::services::auth_service::AuthService;

pub struct ToadAuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for ToadAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService { service }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<
        LocalBoxFuture<'static, Result<Self::Response, Self::Error>>,
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>,
    >;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth = req.headers().get("Authorization");
        if auth.is_none() {
            return Either::Left(Box::pin(async { Err(Error::from(ApiError::Unauthorized)) }));
        }
        let token = auth.unwrap().to_str().unwrap()[7..].trim();
        let data = AuthService::decode_jwt(token);
        if data.is_err() || data.as_ref().unwrap().verifier_id.is_none() {
            error!("Middleware error: failed to decode token");
            return Either::Left(Box::pin(async { Err(Error::from(ApiError::Unauthorized)) }));
        }
        let verifier_id = data.unwrap().verifier_id.unwrap();

        let is_valid_future = AuthService::is_valid_id(verifier_id.clone());
        let pool = req.app_data::<Data<Pool<Postgres>>>().cloned();

        let service_future = self.service.call(req);

        Either::Right(Box::pin(async move {
            let is_valid = is_valid_future.await;
            if !is_valid {
                return Err(Error::from(ApiError::Unauthorized));
            }
            let db_user =
                WalletDao::get_wallet_by_firebase_id(pool.unwrap().as_ref(), verifier_id.clone())
                    .await;
            if db_user.is_none() {
                return Err(Error::from(ApiError::Unauthorized));
            }
            // req.extensions_mut().insert(db_user.unwrap());
            service_future.await
        }))
    }
}
