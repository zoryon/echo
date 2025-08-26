use actix_web::{web, HttpResponse, Responder, HttpRequest, ResponseError};
use diesel::prelude::*;
use chrono::{Utc, Duration};
use rand::rngs::OsRng;
use rand::RngCore;
use uuid::Uuid;

use crate::db::DbPool;
use crate::db::get_conn;
use crate::schema::sessions::dsl::*;
use crate::models::session_models::{CreateSession, Session, SessionResponse, NewSession};

fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub async fn create_session(
    pool: web::Data<DbPool>,
    payload: web::Json<CreateSession>,
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let token_str = generate_token();
    let expiration = Utc::now() + Duration::hours(720); // 30 days

    let new_session = NewSession {
        id: Uuid::new_v4().to_string(),
        user_id: payload.user_id.clone(),
        token: token_str.clone(),
        created_at: Some(Utc::now().naive_utc()),
        expires_at: Some(expiration.naive_utc()),
    };

    match diesel::insert_into(sessions)
        .values(&new_session)
        .execute(&mut conn)
    {
        Ok(_) => HttpResponse::Ok().json(SessionResponse {
            token: token_str,
            expires_at: Some(expiration.naive_utc()),
        }),
        Err(_) => HttpResponse::InternalServerError().body("Failed to create session"),
    }
}


// Get current session info
pub async fn get_current_session(req: HttpRequest, pool: web::Data<DbPool>) -> impl Responder {
    // Assume token comes from "Authorization: Bearer <token>" header
    let auth_header = match req.headers().get("Authorization") {
        Some(h) => h.to_str().unwrap_or(""),
        None => return HttpResponse::Unauthorized().body("Missing token"),
    };

    let token_value = auth_header.strip_prefix("Bearer ").unwrap_or("");

    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let result = sessions
        .filter(token.eq(token_value))
        .first::<Session>(&mut conn);

    match result {
        Ok(session) => HttpResponse::Ok().json(session),
        Err(diesel::result::Error::NotFound) => HttpResponse::Unauthorized().body("Invalid token"),
        Err(_) => HttpResponse::InternalServerError().body("DB error"),
    }
}

// Log out (delete session)
pub async fn delete_current_session(req: HttpRequest, pool: web::Data<DbPool>) -> impl Responder {
    let auth_header = match req.headers().get("Authorization") {
        Some(h) => h.to_str().unwrap_or(""),
        None => return HttpResponse::Unauthorized().body("Missing token"),
    };

    let token_value = auth_header.strip_prefix("Bearer ").unwrap_or("");

    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    match diesel::delete(sessions.filter(token.eq(token_value))).execute(&mut conn) {
        Ok(_) => HttpResponse::Ok().body("Logged out"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to delete session"),
    }
}
