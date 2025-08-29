use actix_web::{web, HttpResponse, Responder, ResponseError};
use actix_web::web::ReqData;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::sql_types::Text;

use crate::db::DbPool;
use crate::db::get_conn;
use crate::models::favorite_models::{NewFavorite, AddFavoriteRequest};
use crate::models::pagination_models::Pagination;
use crate::models::token_models::Claims;
use crate::schema::favorites::dsl as fav_dsl;
use crate::models::song_models::{SongResponse};
use crate::utils::auth_utils::check_ownership;

pub async fn list_favorites(
    pool: web::Data<DbPool>,
    user_id_param: web::Path<String>,
    query: web::Query<Pagination>,
    claims: ReqData<Claims>,
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let user_id: String = user_id_param.into_inner();
    let user_id: &str = match check_ownership(&user_id, &claims) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let pagination = query.into_inner();

    let sql = format!(r#"
        SELECT 
            s.id,
            s.title,
            s.artist_id,
            a.name AS artist_name,
            s.album_id,
            al.name AS album_name,
            s.genre_id,
            g.name AS genre_name,
            s.duration_seconds,
            s.sftp_path,
            s.created_at,
            s.updated_at
        FROM favorites f
        JOIN songs s ON f.song_id = s.id
        JOIN artists a ON s.artist_id = a.id
        LEFT JOIN albums al ON s.album_id = al.id
        LEFT JOIN genres g ON s.genre_id = g.id
        WHERE f.user_id = $1
        {}
    "#, pagination.sql_clause());

    match diesel::sql_query(sql)
        .bind::<Text, _>(user_id)
        .load::<SongResponse>(&mut conn)
    {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn add_favorite(
    pool: web::Data<DbPool>,
    user_id_param: web::Path<String>,
    payload: web::Json<AddFavoriteRequest>,
    claims: ReqData<Claims>,
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let user_id: String = user_id_param.into_inner();
    let user_id: &str = match check_ownership(&user_id, &claims) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let new_fav = NewFavorite {
        user_id: user_id.to_string(),
        song_id: payload.song_id.clone(),
    };

    let result = diesel::insert_into(fav_dsl::favorites)
        .values(&new_fav)
        .execute(&mut conn);

    match result {
        Ok(_) => HttpResponse::Created().finish(),
        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
            HttpResponse::Conflict().body("Song is already in favorites")
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn remove_favorite(
    pool: web::Data<DbPool>,
    path: web::Path<(String, String)>,
    claims: ReqData<Claims>,
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let (user_id_param, song_id_param) = path.into_inner();

    let user_id: String = user_id_param.clone();
    let user_id: &str = match check_ownership(&user_id, &claims) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let affected = diesel::delete(
        fav_dsl::favorites
            .filter(fav_dsl::user_id.eq(user_id))
            .filter(fav_dsl::song_id.eq(song_id_param)),
    )
    .execute(&mut conn);

    match affected {
        Ok(count) if count > 0 => HttpResponse::Ok().finish(),
        Ok(_) => HttpResponse::NotFound().body("Song not found in favorites"),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
