use actix_web::{web, Responder, HttpResponse, ResponseError};
use diesel::prelude::*;
use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::models::user_models::{UpdateUser, UserResponse};
use crate::db::DbPool;
use crate::db::get_conn;
use crate::schema::users::dsl::*;

#[allow(dead_code)]
#[derive(Queryable)]
struct UserRow {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub avatar_url: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

pub async fn update_user(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
    payload: web::Json<UpdateUser>,
) -> impl Responder {
    let user_id_param = path.into_inner(); // ✅ Extract value
    
    // Validate UUID format (ids are stored as strings)
    if Uuid::parse_str(&user_id_param).is_err() { 
        return HttpResponse::BadRequest().body("Invalid UUID");
    }

    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let update_data = payload.into_inner();

    // Use AsChangeset to update only provided fields; None fields are skipped
    let res = diesel::update(users.filter(id.eq(&user_id_param)))
        .set(&update_data)
        .execute(&mut conn);
    match res {
        Ok(affected) if affected > 0 => {
            // fetch back
            match users
                .select((id, username, password_hash, avatar_url, created_at, updated_at))
                .filter(id.eq(&user_id_param))
                .first::<UserRow>(&mut conn) {
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
                Err(_) => HttpResponse::Ok().finish(),
            }
        }
        Ok(_) => HttpResponse::NotFound().body("User not found"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to update user"),
    }
}