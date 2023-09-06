use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::LocalBoxFuture;
use log::error;
use reqwest::header::HeaderName;
use std::future::{ready, Ready};

use crate::errors::ApiError;

pub struct AdminAuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AdminAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AdminAuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AdminAuthMiddlewareService { service }))
    }
}

pub struct AdminAuthMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AdminAuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let header = req.headers().get(HeaderName::from_static("user"));
        if header.is_none() || "admin" != header.unwrap().to_str().unwrap().to_string() {
            error!("Unauthorized request");
            return Box::pin(async { Err(Error::from(ApiError::Unauthorized)) });
        }
        Box::pin(self.service.call(req))
    }
}
