use actix_web::{web, HttpResponse, Responder, ResponseError};
use diesel::result::DatabaseErrorKind;
use diesel::QueryDsl;
use diesel::prelude::*;
use uuid::Uuid;
use diesel::sql_types::Text;

use crate::models::album_models::NewAlbum;
use crate::models::album_models::UpdateAlbum;
use crate::models::pagination_models::Pagination;
use crate::models::song_models::SongResponse;
use crate::{db::{get_conn, DbPool}, models::{album_models::{Album, AlbumQuery}}, utils::pagination_utils::validate_pagination};
use crate::schema::albums::dsl::*;

pub async fn list_albums(
    pool: web::Data<DbPool>,
    query: web::Query<AlbumQuery>,
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let term = query.q.clone().unwrap_or_default().to_lowercase();
    let pagination = &query.pagination;

    let (limit, offset) = match validate_pagination(pagination) {
        Ok(v) => v,
        Err(e) => return e.error_response(),
    };

    let mut query_builder = albums.into_boxed();

    if !term.is_empty() {
        query_builder = query_builder.filter(name.like(format!("%{}%", term)));
    }

    let result = query_builder
        .select(Album::as_select())
        .limit(limit)
        .offset(offset)
        .load::<Album>(&mut conn);

    match result {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn create_album(
    pool: web::Data<DbPool>,
    payload: web::Json<NewAlbum>,
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let new_id = Uuid::new_v4().to_string();

    let new_album = NewAlbum {
        id: new_id.clone(),
        name: payload.name.clone(),
        artist_id: payload.artist_id.clone(),
        release_year: payload.release_year,
        cover_url: payload.cover_url.clone(),
    };

    match diesel::insert_into(albums)
        .values(&new_album)
        .execute(&mut conn)
        {
            Ok(_) => {
                // MySQL doesn't support RETURNING; fetch the inserted row by the id we generated.
                match albums.find(&new_id).select(Album::as_select()).first::<Album>(&mut conn) {
                    Ok(album) => HttpResponse::Created().json(album),
                    Err(diesel::result::Error::NotFound) => HttpResponse::InternalServerError().body("Inserted album not found"),
                    Err(e) => HttpResponse::InternalServerError().body(format!("DB error: {}", e)),
                }
            }
            Err(diesel::result::Error::DatabaseError(kind, info)) => {
                match kind {
                    diesel::result::DatabaseErrorKind::ForeignKeyViolation => {
                        HttpResponse::BadRequest().body("Invalid artist_id: does not exist")
                    }
                    diesel::result::DatabaseErrorKind::UniqueViolation => {
                        HttpResponse::Conflict().body("Album ID already exists")
                    }
                    _ => HttpResponse::InternalServerError().body(format!("DB error: {}", info.message()))
                }
            }
            Err(e) => HttpResponse::InternalServerError().body(format!("Unexpected error: {}", e)),
        }
}

pub async fn get_album(
    pool: web::Data<DbPool>,
    path: web::Path<String>, 
) -> impl Responder {
    let album_id = path.into_inner();

    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return HttpResponse::ServiceUnavailable().body(format!("DB connection error: {}", e)),
    };

    let result = albums
        .filter(id.eq(album_id))
        .first::<Album>(&mut conn);

    match result {
        Ok(album) => HttpResponse::Ok().json(album),
        Err(diesel::result::Error::NotFound) => {
            HttpResponse::NotFound().body("Album not found")
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

pub async fn update_album(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
    payload: web::Json<UpdateAlbum>, 
) -> impl Responder {
    let album_id = path.into_inner();
    let update_data = payload.into_inner();

    // Basic validation
    if update_data.name.trim().is_empty() {
        return HttpResponse::BadRequest().body("Album name cannot be empty");
    }

    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return HttpResponse::ServiceUnavailable().body(format!("DB connection error: {}", e)),
    };

    let target = albums.filter(id.eq(&album_id));

    let exec_result = diesel::update(target)
        .set((
            name.eq(update_data.name),
            release_year.eq(update_data.release_year),
            cover_url.eq(update_data.cover_url),
        ))
        .execute(&mut conn);

    match exec_result {
        Ok(0) => HttpResponse::NotFound().body("Album not found"),
        Ok(_) => {
            match albums.filter(id.eq(&album_id)).select(Album::as_select()).first::<Album>(&mut conn) {
                Ok(updated_album) => HttpResponse::Ok().json(updated_album),
                Err(diesel::result::Error::NotFound) => HttpResponse::NotFound().body("Album not found"),
                Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
            }
        }
        Err(diesel::result::Error::DatabaseError(kind, info)) => {
            match kind {
                diesel::result::DatabaseErrorKind::ForeignKeyViolation => {
                    HttpResponse::BadRequest().body("Invalid artist_id: does not exist")
                }
                _ => HttpResponse::InternalServerError().body(format!("DB error: {}", info.message()))
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

pub async fn delete_album(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> impl Responder {
    let album_id = path.into_inner();

    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return HttpResponse::ServiceUnavailable().body(format!("DB connection error: {}", e)),
    };

    let target = albums.filter(id.eq(&album_id));

    match diesel::delete(target).execute(&mut conn) {
        Ok(0) => HttpResponse::NotFound().body("Album not found"),
        Ok(_) => HttpResponse::Ok().body("Album deleted successfully"),
        Err(diesel::result::Error::DatabaseError(kind, info)) => match kind {
            DatabaseErrorKind::ForeignKeyViolation => {
                HttpResponse::Conflict().body("Cannot delete album: it is referenced by other records")
            }
            _ => HttpResponse::InternalServerError().body(format!("Database error: {}", info.message())),
        },
        Err(e) => HttpResponse::InternalServerError().body(format!("Unexpected error: {}", e)),
    }
}

pub async fn get_album_songs(
    pool: web::Data<DbPool>,
    album_id_param: web::Path<String>,
    query: web::Query<Pagination>,
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let album_id = album_id_param.into_inner();

    let pagination = query.into_inner();
    match validate_pagination(&pagination) {
        Ok(v) => v,
        Err(e) => return e.error_response(),
    };

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
            s.object_url,
            s.created_at,
            s.updated_at
        FROM songs s
        JOIN artists a ON s.artist_id = a.id
        LEFT JOIN albums al ON s.album_id = al.id
        LEFT JOIN genres g ON s.genre_id = g.id
        WHERE s.album_id = $1
        {}
    "#, pagination.sql_clause().unwrap());

    match diesel::sql_query(sql)
        .bind::<Text, _>(album_id)
        .load::<SongResponse>(&mut conn)
    {
        Ok(list) => {
            if list.is_empty() {
                HttpResponse::NotFound().body("No songs found for this album")
            } else {
                HttpResponse::Ok().json(list)
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}