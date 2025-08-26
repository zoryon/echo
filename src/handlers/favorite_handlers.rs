use actix_web::{web, HttpResponse, Responder, ResponseError};
use diesel::prelude::*;

use crate::db::DbPool;
use crate::db::get_conn;
use crate::models::favorite_models::NewFavorite;
use crate::schema::favorites::dsl as fav_dsl;
use crate::models::song_models::{Song, SongResponse};
use crate::schema::songs::dsl as songs_dsl;

pub async fn list_favorites(
    pool: web::Data<DbPool>,
    user_id_param: web::Path<String>,
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let user_id = user_id_param.into_inner();

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
            songs_dsl::audio_url,
            songs_dsl::created_at,
            songs_dsl::updated_at,
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
    payload: web::Json<NewFavorite>,
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let new_fav = NewFavorite {
        user_id: user_id_param.into_inner(),
        song_id: payload.song_id.clone(),
    };

    let result = diesel::insert_into(fav_dsl::favorites)
        .values(&new_fav)
        .execute(&mut conn);

    match result {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn remove_favorite(
    pool: web::Data<DbPool>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let (user_id_param, song_id_param) = path.into_inner();

    let result = diesel::delete(fav_dsl::favorites
        .filter(fav_dsl::user_id.eq(user_id_param))
        .filter(fav_dsl::song_id.eq(song_id_param)))
        .execute(&mut conn);

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
