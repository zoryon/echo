use actix_web::{web, HttpResponse, Responder, ResponseError};
use diesel::prelude::*;

use crate::db::DbPool;
use crate::db::get_conn;
use crate::models::playlist_models::NewPlaylist;
use crate::models::playlist_models::NewPlaylistSong;
use crate::models::playlist_models::Playlist;
use crate::models::song_models::{Song, SongResponse};
use crate::schema::{playlists::dsl as playlists_dsl, playlist_songs::dsl as ps_dsl, songs::dsl as songs_dsl};

// --------------------- Playlists ---------------------
pub async fn list_playlists(
    pool: web::Data<DbPool>,
    user_id_param: web::Path<String>
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let result = playlists_dsl::playlists
        .filter(playlists_dsl::user_id.eq(user_id_param.into_inner()))
        .load::<Playlist>(&mut conn);

    match result {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn create_playlist(
    pool: web::Data<DbPool>,
    user_id_param: web::Path<String>,
    payload: web::Json<NewPlaylist>
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let new_playlist = NewPlaylist {
        user_id: user_id_param.into_inner(),
        name: payload.name.clone(),
        description: payload.description.clone(),
        is_public: payload.is_public,
    };

    let result = diesel::insert_into(playlists_dsl::playlists)
        .values(&new_playlist)
        .execute(&mut conn);

    match result {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn get_playlist(
    pool: web::Data<DbPool>,
    path: web::Path<(String, String)>
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let (user_id_param, playlist_id_param) = path.into_inner();

    let result = playlists_dsl::playlists
        .filter(playlists_dsl::user_id.eq(user_id_param))
        .filter(playlists_dsl::id.eq(playlist_id_param))
        .first::<Playlist>(&mut conn);

    match result {
        Ok(p) => HttpResponse::Ok().json(p),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn update_playlist(
    pool: web::Data<DbPool>,
    path: web::Path<(String, String)>,
    payload: web::Json<NewPlaylist>
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let (user_id_param, playlist_id_param) = path.into_inner();

    let result = diesel::update(playlists_dsl::playlists
        .filter(playlists_dsl::user_id.eq(user_id_param))
        .filter(playlists_dsl::id.eq(playlist_id_param)))
        .set((
            playlists_dsl::name.eq(&payload.name),
            playlists_dsl::description.eq(&payload.description),
            playlists_dsl::is_public.eq(payload.is_public),
        ))
        .execute(&mut conn);

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn delete_playlist(
    pool: web::Data<DbPool>,
    path: web::Path<(String, String)>
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let (user_id_param, playlist_id_param) = path.into_inner();

    let result = diesel::delete(playlists_dsl::playlists
        .filter(playlists_dsl::user_id.eq(user_id_param))
        .filter(playlists_dsl::id.eq(playlist_id_param)))
        .execute(&mut conn);

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// --------------------- Songs in Playlist ---------------------
pub async fn list_playlist_songs(
    pool: web::Data<DbPool>,
    path: web::Path<(String, String)>
) -> impl Responder {
    let ( _user_id_param, playlist_id_param ) = path.into_inner();
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let result = ps_dsl::playlist_songs
        .inner_join(songs_dsl::songs.on(ps_dsl::song_id.eq(songs_dsl::id)))
        .filter(ps_dsl::playlist_id.eq(playlist_id_param))
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

pub async fn add_song_to_playlist(
    pool: web::Data<DbPool>,
    path: web::Path<(String, String)>,
    payload: web::Json<NewPlaylistSong>
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let (_user_id_param, playlist_id_param) = path.into_inner();

    let new_song = NewPlaylistSong {
        playlist_id: playlist_id_param,
        song_id: payload.song_id.clone(),
        position: payload.position,
    };

    let result = diesel::insert_into(ps_dsl::playlist_songs)
        .values(&new_song)
        .execute(&mut conn);

    match result {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn remove_song_from_playlist(
    pool: web::Data<DbPool>,
    path: web::Path<(String, String, String)>
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let (_user_id_param, playlist_id_param, song_id_param) = path.into_inner();

    let result = diesel::delete(ps_dsl::playlist_songs
        .filter(ps_dsl::playlist_id.eq(playlist_id_param))
        .filter(ps_dsl::song_id.eq(song_id_param)))
        .execute(&mut conn);

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
