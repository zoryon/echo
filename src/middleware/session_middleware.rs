use actix_web::{
    dev::{forward_ready, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use actix_web::web::Data;
use futures::future::{LocalBoxFuture, ready, Ready};
use diesel::prelude::*;
use crate::{db::DbPool, models::session_models::Session};
use crate::schema::sessions::dsl::*;
use crate::schema::users::dsl::*;
use crate::models::user_models::User;

pub struct SessionMiddleware;

impl<S, B> Transform<S, ServiceRequest> for SessionMiddleware
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = SessionMiddlewareMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SessionMiddlewareMiddleware { service: Arc::new(service) }))
    }
}

use std::sync::Arc;
pub struct SessionMiddlewareMiddleware<S> {
    service: Arc<S>,
}

impl<S, B> actix_web::dev::Service<ServiceRequest> for SessionMiddlewareMiddleware<S>
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        let pool = req.app_data::<Data<DbPool>>().unwrap().clone();
        let path = req.path().to_string();
        let method = req.method().clone();

        Box::pin(async move {
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("");

            let token_value = auth_header.strip_prefix("Bearer ").unwrap_or("");

            let mut conn = pool.get().map_err(|_| actix_web::error::ErrorInternalServerError("DB error"))?;

            // Check if token exists in DB
            let session_result = sessions
                .filter(crate::schema::sessions::dsl::token.eq(token_value))
                .first::<Session>(&mut conn)
                .optional()
                .map_err(|_| actix_web::error::ErrorInternalServerError("DB error"))?;

            // Allow unauthenticated health checks on GET /health
            if path == "/health" && method == actix_web::http::Method::GET {
                return service.call(req).await;
            }

            // Routes that must be accessed only if logged out
            if path == "/api/sessions" && method == actix_web::http::Method::POST {
                if session_result.is_some() {
                    return Err(actix_web::error::ErrorForbidden("Already logged in"));
                }
                return service.call(req).await;
            }

            // Routes that require login
            if session_result.is_none() {
                return Err(actix_web::error::ErrorUnauthorized("No active session"));
            }

            let session = session_result.unwrap();

            // Attach user_id to request extensions for handlers
            req.extensions_mut().insert(session.clone()); // session.1 = user_id

            // Admin-only check for POST /users
            if path == "/users" && method == actix_web::http::Method::POST {
                let user_data = users
                    .filter(crate::schema::users::dsl::id.eq(session.user_id.clone()))
                    .first::<User>(&mut conn)
                    .map_err(|_| actix_web::error::ErrorInternalServerError("DB error"))?;

                if !user_data.is_admin {
                    return Err(actix_web::error::ErrorForbidden("Admin only"));
                }
            }

            service.call(req).await
        })
    }
}
