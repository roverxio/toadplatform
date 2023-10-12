use actix_web::web::Data;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{Either, LocalBoxFuture};
use log::{debug, error};
use sqlx::{Pool, Postgres};
use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::sync::Arc;

use crate::db::dao::WalletDao;
use crate::errors::errors::ApiError;
use crate::services::auth_service::AuthService;

pub struct ToadAuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for ToadAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Arc::new(service),
        }))
    }
}

#[derive(Clone)]
pub struct AuthMiddlewareService<S> {
    service: Arc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
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
        let fut = async move {
            // Check for the "Authorization" header
            let auth = req.headers().get("Authorization");
            if auth.is_none() {
                return Err(Error::from(ApiError::Unauthorized));
            }

            // Extract and decode the JWT token
            let token = auth.and_then(|value| value.to_str().ok()).and_then(|s| {
                if s.starts_with("Bearer ") {
                    Some(s[7..].trim())
                } else {
                    None
                }
            });
            if token.is_none() {
                error!("Middleware error: failed to extract token");
                return Err(Error::from(ApiError::Unauthorized));
            }
            let data = AuthService::decode_jwt(token.unwrap());
            if data.is_err() || data.as_ref().unwrap().verifier_id.is_none() {
                error!("Middleware error: failed to decode token");
                return Err(Error::from(ApiError::Unauthorized));
            }

            let verifier_id = data.unwrap().verifier_id.unwrap();

            // Validate verifier_id
            let user = AuthService::is_valid_id(verifier_id.clone()).await;
            if user.is_none() {
                return Err(Error::from(ApiError::Unauthorized));
            }

            // Fetch user from the database
            let pool = req.app_data::<Data<Pool<Postgres>>>().cloned();
            let mut db_user;
            db_user = WalletDao::get_wallet_by_external_user_id(
                pool.unwrap().as_ref(),
                verifier_id.clone(),
            )
            .await;
            if db_user.is_none() {
                debug!("Probably a new user. Not found on db, but exists on firebase");
                db_user = Some(crate::db::dao::User::from(user.unwrap()));
            }

            // Insert db_user into req's extensions
            req.extensions_mut().insert(db_user.unwrap());

            Ok(req)
        };

        let service = self.service.clone();
        Either::Right(Box::pin(async move {
            match fut.await {
                Ok(validated_req) => service.call(validated_req).await,
                Err(e) => Err(e),
            }
        }))
    }
}
