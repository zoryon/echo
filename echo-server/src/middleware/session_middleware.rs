use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::Method,
    web::Data,
    Error, HttpMessage,
};
use diesel::prelude::*;
use futures::future::{ready, LocalBoxFuture, Ready};
use std::sync::Arc;
use regex::Regex;
use crate::{
    constants::middleware_constants::ADMIN_ONLY_ROUTES, db::DbPool, models::{session_models::Session, token_models::Claims, user_models::PublicUser}, schema::{sessions, users}, utils::token_utils::verify_jwt
};

pub struct SessionMiddlewareFactory;

impl<S, B> Transform<S, ServiceRequest> for SessionMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = SessionMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SessionMiddleware {
            service: Arc::new(service),
        }))
    }
}
pub struct SessionMiddleware<S> {
    service: Arc<S>,
}

impl<S, B> Service<ServiceRequest> for SessionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        
        let pool_option = req.app_data::<Data<DbPool>>().cloned();
        let path = req.path().to_string();
        let method = req.method().clone();
        
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("")
            .to_string();

        Box::pin(async move {
            if path == "/health" && method == Method::GET {
                return service.call(req).await;
            }

            let token_value = auth_header.strip_prefix("Bearer ").unwrap_or("");
            
            let pool = pool_option.ok_or_else(|| actix_web::error::ErrorInternalServerError("Database pool not configured"))?;
            let mut conn = pool.get().map_err(|_| actix_web::error::ErrorInternalServerError("Could not get DB connection"))?;

            let secret = std::env::var("JWT_SECRET")
                .map_err(|_| actix_web::error::ErrorInternalServerError("Missing JWT_SECRET configuration"))?;

            // REQUIREMENT 1: Verify the JWT and save the claims.
            let claims: Option<Claims> = verify_jwt(token_value, secret.as_bytes());
            
            // REQUIREMENT 2: Check if the token is in the database.
            // This is done only if the claims from the JWT were successfully verified.
            let session_result = if claims.is_some() {
                sessions::table
                    .filter(sessions::token.eq(token_value))
                    .first::<Session>(&mut conn)
                    .optional()
                    .map_err(|_| actix_web::error::ErrorInternalServerError("Database error checking session"))?
            } else {
                None
            };

            // This block runs only if the user has a valid, active session.
            if let Some(session) = session_result {
                // Forbid access to "logged-out only" routes.
                if path == "/api/sessions" && method == Method::POST {
                    return Err(actix_web::error::ErrorForbidden("Already logged in"));
                }

                // REQUIREMENT 3: Upload claims for other functions to use.
                // It is safe to unwrap claims here because we know it is `Some`.
                req.extensions_mut().insert(claims.unwrap());

                // You can also insert the session object if handlers need it.
                req.extensions_mut().insert(session.clone());

                let requires_admin: bool = ADMIN_ONLY_ROUTES.iter().any(|(route_path, route_method)| {
                    if route_method != &method { return false; }

                    // Convert route template to regex, e.g., /api/songs/{song_id} -> ^/api/songs/[^/]+$
                    let regex_pattern: Regex = Regex::new(
                        &format!("^{}$", regex::escape(route_path).replace(r"\{song_id\}", r"[^/]+"))
                    ).unwrap();

                    regex_pattern.is_match(&path)
                });

                if requires_admin {
                    let user_data: PublicUser = users::table
                        .select(PublicUser::as_select())
                        .filter(users::id.eq(session.user_id))
                        .first::<PublicUser>(&mut conn)
                        .map_err(|_| actix_web::error::ErrorInternalServerError("Could not query user data"))?;

                    if !user_data.is_admin {
                        return Err(actix_web::error::ErrorForbidden("This action requires admin privileges"));
                    }
                }

                // If all checks pass, forward the request to the handler.
                return service.call(req).await;
            }

            // This section handles all cases where the user is NOT properly authenticated
            // (no token, invalid token, or token not in DB).

            // Allow access to public routes like login.
            if path == "/api/sessions" && method == Method::POST {
                return service.call(req).await;
            }

            // For all other routes, deny access.
            Err(actix_web::error::ErrorUnauthorized("A valid session is required to access this resource"))
        })
    }
}