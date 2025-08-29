use actix_web::{web, HttpResponse, Responder, ResponseError};
use actix_multipart::Multipart;
use diesel::prelude::*;
use diesel::sql_types::Text;
use futures::StreamExt;
use futures::TryStreamExt;
use uuid::Uuid;

use crate::db::DbPool;
use crate::db::get_conn;
use crate::models::pagination_models::Pagination;
use crate::models::song_models::{Song, SongResponse, NewSong, UpdateSong};
use crate::schema::songs::dsl::*;
use crate::utils::audio_utils::normalize_song_async;
use crate::utils::pagination_utils::validate_pagination;

pub async fn list_songs(
    pool: web::Data<DbPool>,
    query: web::Query<Pagination>,
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

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
        {}
    "#, pagination.sql_clause().unwrap());

    match diesel::sql_query(sql).load::<SongResponse>(&mut conn) {
        Ok(list) => HttpResponse::Ok().json(list),
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

    let song_id: String = song_id_param.into_inner();

    let sql = r#"
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
        WHERE s.id = $1
    "#;

    match diesel::sql_query(sql)
        .bind::<Text, _>(song_id)
        .load::<SongResponse>(&mut conn)
    {
        Ok(mut list) => {
            if let Some(song) = list.pop() {
                HttpResponse::Ok().json(song)
            } else {
                HttpResponse::NotFound().finish()
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn stream_song(
    pool: web::Data<DbPool>,
    song_id_param: web::Path<String>
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let song_id = song_id_param.into_inner();
    let song_record = songs.filter(id.eq(song_id))
        .first::<crate::models::song_models::Song>(&mut conn);

    match song_record {
        Ok(song) => {
            HttpResponse::Found() // 302 redirect
                .append_header(("Location", song.object_url)) 
                .finish()
        },
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn create_one_or_more_songs(
    pool: web::Data<DbPool>,
    mut payload: Multipart,
) -> impl Responder {
    // Temp storage for each song
    struct SongData {
        file: Vec<u8>,
        metadata: NewSong,
    }
    let mut songs_batch: Vec<SongData> = Vec::new();

    let mut current_file: Option<Vec<u8>> = None;
    let mut current_meta: Option<NewSong> = None;

    while let Ok(Some(mut field)) = payload.try_next().await {
        let name = field.content_disposition().unwrap().get_name().unwrap();

        if name == "file" {
            let mut buf = Vec::new();
            while let Some(chunk) = field.next().await {
                match chunk {
                    Ok(data) => buf.extend_from_slice(&data),
                    Err(e) => {
                        return HttpResponse::InternalServerError()
                            .body(format!("Failed to read file chunk: {}", e));
                    }
                }
            }
            current_file = Some(buf);
        } else if name == "metadata" {
            let mut bytes = web::BytesMut::new();
            while let Some(chunk) = field.next().await {
                match chunk {
                    Ok(data) => bytes.extend_from_slice(&data),
                    Err(e) => {
                        return HttpResponse::InternalServerError()
                            .body(format!("Failed to read metadata chunk: {}", e));
                    }
                }
            }
            let meta_result: Result<NewSong, _> = serde_json::from_slice(&bytes);
            match meta_result {
                Ok(meta) => current_meta = Some(meta),
                Err(e) => {
                    return HttpResponse::BadRequest()
                        .body(format!("Invalid metadata JSON: {}", e));
                }
            }
        }

        // First, check if both parts are available without consuming them
        if current_file.is_some() && current_meta.is_some() {
            // Now that we know they exist, we can safely take them.
            // .unwrap() is safe here because we just checked with is_some().
            let file = current_file.take().unwrap();
            let meta = current_meta.take().unwrap();
            
            songs_batch.push(SongData { file, metadata: meta });
        }
    }

    if songs_batch.is_empty() || songs_batch.len() > 10 {
        return HttpResponse::BadRequest()
            .body("You must upload between 1 and 10 songs per request");
    }

    let write_base: String = std::env::var("OBJECT_STORAGE_WRITE_BASE_URL")
        .expect("OBJECT_STORAGE_WRITE_BASE_URL not set");
    let read_base: String = std::env::var("OBJECT_STORAGE_READ_BASE_URL")
        .expect("OBJECT_STORAGE_READ_BASE_URL not set");

    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    // Process each song sequentially (or in parallel with join_all)
    // Return inserted IDs (Vec<String>) which are serializable by serde
    let mut results: Vec<String> = Vec::new();
    for mut song in songs_batch {
        song.metadata.id = Uuid::new_v4().to_string();

        // Normalize
        let normalized_file = match normalize_song_async(&song.file).await {
            Ok(f) => f,
            Err(e) => {
                return HttpResponse::InternalServerError().body(format!("Audio normalization error: {}", e));
            }
        };

        // Upload
        let object_name = format!("{}.mp3", song.metadata.id);
        let upload_url = format!("{}/{}", write_base, object_name);
        let client = reqwest::Client::new();
        let res = match client
            .put(&upload_url)
            .header("Content-Type", "audio/mpeg")
            .body(normalized_file)
            .send()
            .await
        {
            Ok(r) => r,
            Err(_) => {
                return HttpResponse::InternalServerError().body("Failed to upload");
            }
        };

        if !res.status().is_success() {
            return HttpResponse::InternalServerError().body("Failed to upload to Object Storage");
        }

        song.metadata.object_url = format!("{}/{}", read_base, object_name);

        // Insert DB
        if let Err(_) = diesel::insert_into(songs)
            .values(&song.metadata)
            .execute(&mut conn)
        {
            return HttpResponse::InternalServerError().finish();
        }

        // Collect result as the new song ID
        results.push(song.metadata.id.clone());
    }

    HttpResponse::Created().json(results)
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

    let song_id  = song_id_param.into_inner();

    // Fetch the song to get its Object URL
    let song_record = match songs.filter(id.eq(&song_id))
        .first::<crate::models::song_models::Song>(&mut conn) 
    {
        Ok(s) => s,
        Err(_) => return HttpResponse::NotFound().finish(),
    };

    // Delete object from Object Storage via signed URL
    let client = reqwest::Client::new();
    let res = client
        .delete(&song_record.object_url)
        .send()
        .await;

    if res.is_err() || !res.as_ref().unwrap().status().is_success() {
        return HttpResponse::InternalServerError()
            .body("Failed to delete object from storage");
    }
    
    // Delete DB record
    let deleted = diesel::delete(songs.filter(id.eq(song_id)))
        .execute(&mut conn);

    match deleted {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
