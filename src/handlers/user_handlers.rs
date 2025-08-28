use actix_web::web::ReqData;
use actix_web::{web, Responder, HttpResponse, ResponseError};
use diesel::prelude::*;
use uuid::Uuid;
use bcrypt::hash;

use crate::models::user_models::{CreateUser, NewUser, UpdateUser, User, UserResponse, UserRow};
use crate::db::DbPool;
use crate::db::get_conn;
use crate::schema::users::dsl::*;
use crate::models::token_models::Claims;
use crate::utils::auth_utils::check_ownership;

pub async fn update_user(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
    payload: web::Json<UpdateUser>,
    claims: ReqData<Claims>,
) -> impl Responder {
    let user_id: String = path.into_inner();
    let user_id: &str = match check_ownership(&user_id, &claims) {
        Ok(valid_id) => valid_id,
        Err(resp) => return resp,
    };
    
    // Validate UUID format (ids are stored as strings)
    if Uuid::parse_str(&user_id).is_err() {
        return HttpResponse::BadRequest().body("Invalid UUID");
    }

    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let update_data = payload.into_inner();

    // Update only provided fields
    match diesel::update(users.filter(id.eq(user_id)))
        .set(&update_data)
        .execute(&mut conn)
    {
        Ok(affected) if affected > 0 => {
            // Fetch updated user
            match users
                .select((id, username, password_hash, avatar_url, created_at, updated_at))
                .filter(id.eq(user_id))
                .first::<UserRow>(&mut conn)
            {
                Ok(row) => {
                    let response = UserResponse {
                        id: Uuid::parse_str(&row.id).unwrap_or_else(|_| Uuid::nil()),
                        username: row.username,
                        avatar_url: row.avatar_url,
                        created_at: row.created_at,
                        updated_at: row.updated_at,
                    };
                    HttpResponse::Ok().json(response)
                }
                Err(_) => HttpResponse::InternalServerError().body("Failed to fetch updated user"),
            }
        }
        Ok(_) => HttpResponse::NotFound().body("User not found"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to update user"),
    }
}

pub async fn create_user(
    pool: web::Data<DbPool>,
    payload: web::Json<CreateUser>,
    claims: ReqData<Claims>,
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let user_id: &str = &claims.sub;

    // Load current user to verify is_admin
    let current_user: User = match users
        .filter(id.eq(user_id.to_string()))
        .first::<User>(&mut conn) {
        Ok(u) => u,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to load current user"),
    };

    if !current_user.is_admin {
        return HttpResponse::Forbidden().body("This action requires admin privileges");
    }

    let data = payload.into_inner();

    if data.username.trim().is_empty() || data.password.is_empty() {
        return HttpResponse::BadRequest().body("username and password are required");
    }

    // Basic uniqueness check
    if let Ok::<User, _>(_existing) = users
        .filter(username.eq(&data.username))
        .first::<User>(&mut conn) {
        return HttpResponse::Conflict().body("username already exists");
    }

    let uid = Uuid::new_v4().to_string();
    let pwd_hash = match hash(&data.password, bcrypt::DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to hash password"),
    };

    let new_user = NewUser {
        id: uid.clone(),
        username: data.username,
        password_hash: pwd_hash,
        avatar_url: data.avatar_url,
        is_admin: data.is_admin.unwrap_or(false),
    };

    match diesel::insert_into(users).values(&new_user).execute(&mut conn) {
        Ok(_) => {
            // Return minimal safe response
            HttpResponse::Created().json(serde_json::json!({
                "id": uid,
                "username": new_user.username,
                "avatar_url": new_user.avatar_url,
                "is_admin": new_user.is_admin
            }))
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to create user"),
    }
}