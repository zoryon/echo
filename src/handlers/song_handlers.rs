use actix_web::{web, HttpResponse, Responder, ResponseError};
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::DbPool;
use crate::db::get_conn;
use crate::models::song_models::{Song, SongResponse, NewSong, UpdateSong};
use crate::schema::songs::dsl::*;

pub async fn list_songs(pool: web::Data<DbPool>) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let result = songs.load::<Song>(&mut conn);

    match result {
        Ok(song_list) => {
            let items: Vec<SongResponse> = song_list.into_iter().map(SongResponse::from).collect();
            HttpResponse::Ok().json(items)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn get_song(
    pool: web::Data<DbPool>,
    song_id_param: web::Path<String>
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let song_id_param = song_id_param.into_inner();

    let result = songs
        .filter(id.eq(song_id_param))
        .first::<Song>(&mut conn);

    match result {
        Ok(song) => HttpResponse::Ok().json(SongResponse::from(song)),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn create_song(
    pool: web::Data<DbPool>,
    payload: web::Json<NewSong>
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let mut new_song = payload.into_inner();
    new_song.id = Uuid::new_v4().to_string();

    let inserted = diesel::insert_into(songs)
        .values(&new_song)
        .execute(&mut conn);

    match inserted {
        Ok(_) => {
            // fetch created row
            match songs.filter(id.eq(new_song.id.clone())).first::<Song>(&mut conn) {
                Ok(created) => HttpResponse::Created().json(SongResponse::from(created)),
                Err(_) => HttpResponse::Created().finish(),
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn update_song(
    pool: web::Data<DbPool>,
    song_id_param: web::Path<String>,
    payload: web::Json<UpdateSong>
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let song_id_param = song_id_param.into_inner();

    let updated = diesel::update(songs.filter(id.eq(song_id_param.clone())))
        .set(&*payload)
        .execute(&mut conn);

    match updated {
        Ok(_) => {
            match songs.filter(id.eq(song_id_param)).first::<Song>(&mut conn) {
                Ok(updated_song) => HttpResponse::Ok().json(SongResponse::from(updated_song)),
                Err(_) => HttpResponse::Ok().finish(),
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn delete_song(
    pool: web::Data<DbPool>,
    song_id_param: web::Path<String>
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let song_id_param = song_id_param.into_inner();

    let deleted = diesel::delete(songs.filter(id.eq(song_id_param)))
        .execute(&mut conn);

    match deleted {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
