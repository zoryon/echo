use actix_web::web::ReqData;
use actix_web::{web, HttpResponse, Responder, ResponseError};
use diesel::prelude::*;
use diesel::sql_types::Text;
use uuid::Uuid;
use diesel::result::{DatabaseErrorKind, Error as DieselError};

use crate::db::DbPool;
use crate::db::get_conn;
use crate::models::pagination_models::Pagination;
use crate::models::playlist_models::{NewPlaylist, NewPlaylistSong, AddSongRequest};
use crate::models::playlist_models::{Playlist, PlaylistQuery};
use crate::models::song_models::{SongResponse};
use crate::models::token_models::Claims;
use crate::schema::{playlists::dsl as playlists_dsl, playlist_songs::dsl as ps_dsl};
use crate::utils::auth_utils::check_ownership;
use crate::utils::pagination_utils::validate_pagination;

// --------------------- Playlists ---------------------
pub async fn list_playlists(
    pool: web::Data<DbPool>,
    user_id_param: web::Path<String>,
    query: web::Query<PlaylistQuery>,
    claims: ReqData<Claims>,
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let user_id: String = user_id_param.into_inner();
    let logged_in_user_id: &String = &claims.sub;
    let is_owner: bool = logged_in_user_id == &user_id;

    // Pagination
    let pagination = Pagination {
        limit: query.limit,
        offset: query.offset,
    };
    let (limit, offset) = match validate_pagination(&pagination) {
        Ok(v) => v,
        Err(e) => return e.error_response(),
    };

    // Base query: playlists of this user
    let mut q = playlists_dsl::playlists
        .filter(playlists_dsl::user_id.eq(&user_id))
        .into_boxed();

    // Only public if not owner
    if !is_owner {
        q = q.filter(playlists_dsl::is_public.eq(true));
    }

    // Optional filtering by name
    if let Some(ref name) = query.name {
        let like_pattern = format!("%{}%", name);
        q = q.filter(playlists_dsl::name.like(like_pattern));
    }

    // Pagination + exec
    let result = q.limit(limit).offset(offset).load::<Playlist>(&mut conn);

    match result {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn create_playlist(
    pool: web::Data<DbPool>,
    user_id_param: web::Path<String>,
    payload: web::Json<NewPlaylist>,
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

    let new_playlist = NewPlaylist {
        user_id: user_id.to_string(),
        name: payload.name.clone(),
        description: payload.description.clone(),
        is_public: payload.is_public,
        id: Uuid::new_v4().to_string(),
    };

    let result = diesel::insert_into(playlists_dsl::playlists)
        .values(&new_playlist)
        .execute(&mut conn);

    match result {
        Ok(_) => HttpResponse::Created().finish(),
        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
            HttpResponse::Conflict().body("Playlist already exists")
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn get_playlist(
    pool: web::Data<DbPool>,
    path: web::Path<(String, String)>,
    claims: ReqData<Claims>,
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let (user_id_param, playlist_id_param) = path.into_inner();

    let user_id: String = user_id_param.clone();
    let logged_in_user_id: &String = &claims.sub;

    let is_owner: bool = logged_in_user_id == &user_id;
    
    // Building a query to find the specific playlist for the user.
    // Use .into_boxed() to allow for conditionally adding more filters.
    let mut query = playlists_dsl::playlists
        .filter(playlists_dsl::id.eq(&playlist_id_param))
        .filter(playlists_dsl::user_id.eq(&user_id_param))
        .into_boxed();

    // If the person making the request is NOT the owner, they can only
    // see the playlist if it's public. We add this as a required condition.
    if !is_owner {
        query = query.filter(playlists_dsl::is_public.eq(true));
    }

    let result = query.first::<Playlist>(&mut conn);

    match result {
        Ok(p) => HttpResponse::Ok().json(p),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn update_playlist(
    pool: web::Data<DbPool>,
    path: web::Path<(String, String)>,
    payload: web::Json<NewPlaylist>,
    claims: ReqData<Claims>,
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let (user_id_param, playlist_id_param) = path.into_inner();

    let user_id: String = user_id_param.clone();
    let user_id: &str = match check_ownership(&user_id, &claims) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let result = diesel::update(playlists_dsl::playlists
        .filter(playlists_dsl::user_id.eq(user_id))
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
    path: web::Path<(String, String)>,
    claims: ReqData<Claims>,
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let (user_id_param, playlist_id_param) = path.into_inner();

    let user_id: String = user_id_param.clone();
    let user_id: &str = match check_ownership(&user_id, &claims) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let affected: Result<usize, DieselError>  = diesel::delete(playlists_dsl::playlists
        .filter(playlists_dsl::user_id.eq(user_id))
        .filter(playlists_dsl::id.eq(playlist_id_param)))
        .execute(&mut conn);

    match affected {
        Ok(0) => HttpResponse::NotFound().body("Playlist not found"),
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// --------------------- Songs in Playlist ---------------------
pub async fn list_playlist_songs(
    pool: web::Data<DbPool>,
    path: web::Path<(String, String)>,
    query: web::Query<Pagination>,
    claims: ReqData<Claims>,
) -> impl Responder {
    // Only the playlist_id is needed from the path for the initial query
    let (_user_id_param, playlist_id_param) = path.into_inner();
    let logged_in_user_id = &claims.sub;

    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    // Fetch the playlist by its ID first to check its status (public/private)
    let playlist = match playlists_dsl::playlists
        .find(&playlist_id_param)
        .first::<Playlist>(&mut conn)
        .optional()
    {
        Ok(Some(p)) => p, // playlist found
        Ok(None) => return HttpResponse::NotFound().body("Playlist not found"), // playlist does not exist
        Err(_) => return HttpResponse::InternalServerError().finish(), // database error
    };

    let is_public: bool = playlist.is_public.unwrap_or(false);

    // Authorize access
    // Allow access if the playlist is public OR if the logged-in user is the owner
    let is_owner: bool = playlist.user_id == *logged_in_user_id;
    if !is_public && !is_owner {
        // It's private and the requester is not the owner.
        // Return 404 to avoid revealing its existence.
        return HttpResponse::NotFound().body("Playlist not found");
    }

    // The user is authorized. Proceed to fetch songs
    let pagination: Pagination = query.into_inner();
    if let Err(e) = validate_pagination(&pagination) {
        return e.error_response();
    };
    
    let sql = format!(r#"
        SELECT 
            s.id, s.title, s.artist_id, a.name AS artist_name,
            s.album_id, al.name AS album_name, s.genre_id, g.name AS genre_name,
            s.duration_seconds, s.object_url, s.created_at, s.updated_at
        FROM playlist_songs ps
        JOIN songs s ON ps.song_id = s.id
        JOIN artists a ON s.artist_id = a.id
        LEFT JOIN albums al ON s.album_id = al.id
        LEFT JOIN genres g ON s.genre_id = g.id
        WHERE ps.playlist_id = $1
        {}
    "#, pagination.sql_clause().unwrap_or_default());

    match diesel::sql_query(sql)
        .bind::<Text, _>(&playlist_id_param)
        .load::<SongResponse>(&mut conn) 
    {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn add_song_to_playlist(
    pool: web::Data<DbPool>,
    path: web::Path<(String, String)>,
    payload: web::Json<AddSongRequest>,
    claims: ReqData<Claims>,
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let (user_id_param, playlist_id_param) = path.into_inner();

    let user_id: String = user_id_param.clone();
    let user_id: &str = match check_ownership(&user_id, &claims) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    // Ensure the playlist actually belongs to the user
    let playlist_owned = match playlists_dsl::playlists
        .filter(playlists_dsl::id.eq(&playlist_id_param))
        .filter(playlists_dsl::user_id.eq(user_id))
        .first::<Playlist>(&mut conn)
        .optional() {
            Ok(opt) => opt,
            Err(_) => return HttpResponse::InternalServerError().finish(),
        };

    if playlist_owned.is_none() {
        return HttpResponse::NotFound().body("Playlist not found or not owned by user");
    }

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
        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
            HttpResponse::Conflict().body("Song already in playlist")
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn remove_song_from_playlist(
    pool: web::Data<DbPool>,
    path: web::Path<(String, String, String)>,
    claims: ReqData<Claims>,
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let (user_id_param, playlist_id_param, song_id_param) = path.into_inner();

    let user_id: String = user_id_param.clone();
    let user_id: &str = match check_ownership(&user_id, &claims) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    // Ensure the playlist actually belongs to the user
    let playlist_owned = match playlists_dsl::playlists
        .filter(playlists_dsl::id.eq(&playlist_id_param))
        .filter(playlists_dsl::user_id.eq(user_id))
        .first::<Playlist>(&mut conn)
        .optional() {
            Ok(opt) => opt,
            Err(_) => return HttpResponse::InternalServerError().finish(),
        };

    if playlist_owned.is_none() {
        return HttpResponse::NotFound().body("Playlist not found or not owned by user");
    }

    let affected = diesel::delete(ps_dsl::playlist_songs
        .filter(ps_dsl::playlist_id.eq(playlist_id_param))
        .filter(ps_dsl::song_id.eq(song_id_param)))
        .execute(&mut conn);

    match affected {
        Ok(0) => HttpResponse::NotFound().body("Song not found"),
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
