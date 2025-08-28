use actix_web::{web, HttpResponse, Responder, ResponseError};
use actix_web::web::ReqData;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};

use crate::db::DbPool;
use crate::db::get_conn;
use crate::models::favorite_models::{NewFavorite, AddFavoriteRequest};
use crate::models::token_models::Claims;
use crate::schema::favorites::dsl as fav_dsl;
use crate::models::song_models::{Song, SongResponse};
use crate::schema::songs::dsl as songs_dsl;
use crate::utils::auth_utils::check_ownership;

pub async fn list_favorites(
    pool: web::Data<DbPool>,
    user_id_param: web::Path<String>,
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

    let result = fav_dsl::favorites
        .inner_join(songs_dsl::songs.on(fav_dsl::song_id.eq(songs_dsl::id)))
        .filter(fav_dsl::user_id.eq(user_id))
        .select((
            songs_dsl::id,
            songs_dsl::title,
            songs_dsl::artist_id,
            songs_dsl::album_id,
            songs_dsl::genre,
            songs_dsl::duration_seconds,
            songs_dsl::created_at,
            songs_dsl::updated_at,
            songs_dsl::sftp_path,
        ))
        .load::<Song>(&mut conn);

    match result {
        Ok(list) => {
            let items: Vec<SongResponse> = list.into_iter().map(SongResponse::from).collect();
            HttpResponse::Ok().json(items)
        }
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
