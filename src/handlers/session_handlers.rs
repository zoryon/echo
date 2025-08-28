use actix_web::web::ReqData;
use actix_web::{web, HttpResponse, Responder, ResponseError};
use diesel::prelude::*;
use chrono::{Utc, Duration};
use uuid::Uuid;
use bcrypt::verify;

use crate::db::DbPool;
use crate::db::get_conn;
use crate::models::token_models::Claims;
use crate::models::user_models::User;
use crate::schema::sessions::dsl::*;
use crate::models::session_models::{CreateSession, Session, SessionResponse, NewSession};
use crate::schema::users;
use crate::utils::token_utils::generate_jwt;

pub async fn create_session(
    pool: web::Data<DbPool>,
    payload: web::Json<CreateSession>,
    secret: web::Data<Vec<u8>>,
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let user_result: QueryResult<User> = users::table
        .filter(users::username.eq(&payload.username))
        .select(User::as_select())
        .first(&mut conn);

    let user = match user_result {
        Ok(u) => u,
        Err(diesel::result::Error::NotFound) => {
            return HttpResponse::Unauthorized().body("Invalid credentials")
        }
        Err(_) => return HttpResponse::InternalServerError().body("DB error"),
    };

    // Verify password using bcrypt
    if !verify(&payload.password, &user.password_hash).unwrap_or(false) {
        return HttpResponse::Unauthorized().body("Invalid credentials");
    }

    // Create new session
    let token_str = generate_jwt(&user.id, &secret);
    let expiration = Utc::now() + Duration::hours(720); // 30 days

    let new_session = NewSession {
        id: Uuid::new_v4().to_string(),
        user_id: user.id.clone(),
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
pub async fn get_session(
    session_id_path: web::Path<String>, 
    pool: web::Data<DbPool>,
    claims: ReqData<Claims>,
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let session_id: String = session_id_path.into_inner();
    let logged_user_id: &String = &claims.sub;

    let session_result = sessions
        .filter(id.eq(&session_id))
        .filter(user_id.eq(logged_user_id))
        .first::<Session>(&mut conn)
        .optional();

    match session_result {
        Ok(Some(sess)) => HttpResponse::Ok().json(sess),
        Ok(None) => HttpResponse::NotFound().body("Not Found"),
        Err(_) => HttpResponse::InternalServerError().body("Database error"),
    }
}

// Log out (delete session)
pub async fn delete_session(
    session_id_path: web::Path<String>,
    pool: web::Data<DbPool>,
    claims: ReqData<Claims>,
) -> impl Responder {
    let session_id = session_id_path.into_inner();
    let logged_user_id = &claims.sub;

    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let deleted_count = diesel::delete(
        sessions.filter(id.eq(&session_id)).filter(user_id.eq(logged_user_id))
    ).execute(&mut conn);

    match deleted_count {
        Ok(0) => HttpResponse::NotFound().body("Not Found"),
        Ok(_) => HttpResponse::Ok().body("Session deleted"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to delete session"),
    }
}
