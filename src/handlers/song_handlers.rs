use std::fs::File;
use std::io::Write;

use actix_web::{web, HttpResponse, Responder, ResponseError};
use actix_multipart::Multipart;
use diesel::prelude::*;
use futures::StreamExt;
use futures::TryStreamExt;
use uuid::Uuid;

use crate::db::DbPool;
use crate::db::get_conn;
use crate::models::song_models::{Song, SongResponse, NewSong, UpdateSong};
use crate::schema::songs::dsl::*;
use crate::utils::file_utils::{upload_file_sftp, delete_file_sftp, stream_song_sftp};

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

pub async fn stream_song(
    pool: web::Data<DbPool>,
    song_id_param: web::Path<String>
) -> impl Responder {
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let song_id_param = song_id_param.into_inner();
    let song_record = songs.filter(id.eq(song_id_param))
        .first::<crate::models::song_models::Song>(&mut conn);

    match song_record {
        Ok(song) => {
            // stream_song_sftp returns HttpResponse with streaming body
            stream_song_sftp(song.sftp_path).await
        },
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn create_song(
    pool: web::Data<DbPool>,
    mut payload: Multipart,
) -> impl Responder {
    let mut new_song: Option<NewSong> = None;
    let mut temp_file_path: Option<String> = None;

    // Iterate multipart fields
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition().unwrap();
        let name = content_disposition.get_name().unwrap();

        if name == "file" {
            // Save file temporarily
            let file_id = Uuid::new_v4().to_string();
            let file_path = format!("/tmp/{}", file_id);
            let mut f = File::create(&file_path).unwrap();

            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                f.write_all(&data).unwrap();
            }

            temp_file_path = Some(file_path);
        } else if name == "metadata" {
            // Parse JSON metadata
            let mut bytes = web::BytesMut::new();
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                bytes.extend_from_slice(&data);
            }
            new_song = Some(serde_json::from_slice::<NewSong>(&bytes).unwrap());
        }
    }

    if new_song.is_none() || temp_file_path.is_none() {
        return HttpResponse::BadRequest().body("Missing file or metadata");
    }

    let mut new_song = new_song.unwrap();
    new_song.id = Uuid::new_v4().to_string();
    new_song.sftp_path = format!("/var/www/songs/{}.mp3", new_song.id); // path on VM

    // Upload via SFTP
    if let Err(e) = upload_file_sftp(&temp_file_path.unwrap(), &new_song.sftp_path) {
        return HttpResponse::InternalServerError().body(format!("SFTP error: {}", e));
    }

    // Insert into DB
    let mut conn = match get_conn(&pool) {
        Ok(c) => c,
        Err(e) => return e.error_response(),
    };

    let inserted = diesel::insert_into(songs)
        .values(&new_song)
        .execute(&mut conn);

    match inserted {
        Ok(_) => {
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

    // 1️⃣ Fetch the song to get its sftp_path
    let song_record = match songs.filter(id.eq(&song_id_param))
        .first::<crate::models::song_models::Song>(&mut conn) 
    {
        Ok(s) => s,
        Err(_) => return HttpResponse::NotFound().finish(),
    };

    // 2️⃣ Delete file from VM via SFTP
    if let Err(e) = delete_file_sftp(&song_record.sftp_path) {
        return HttpResponse::InternalServerError()
            .body(format!("Failed to delete remote file: {}", e));
    }

    // 3️⃣ Delete DB record
    let deleted = diesel::delete(songs.filter(id.eq(song_id_param)))
        .execute(&mut conn);

    match deleted {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
