use actix_web::web;

use crate::handlers::song_handlers::{
    list_songs, get_song, create_song, update_song, delete_song, stream_song
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/songs")
            .route("", web::get().to(list_songs))
            .route("", web::post().to(create_song))
            .route("/{song_id}", web::get().to(get_song))
            .route("/{song_id}", web::put().to(update_song))
            .route("/{song_id}", web::delete().to(delete_song))
            .route("/{song_id}/stream", web::get().to(stream_song))
    );
}
