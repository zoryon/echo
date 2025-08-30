use actix_web::web;

use crate::handlers::album_handlers::{
    list_albums, create_album, get_album, update_album, delete_album, get_album_songs
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/albums")
            .route("", web::get().to(list_albums))
            .route("", web::post().to(create_album))
            .route("/{album_id}", web::get().to(get_album))
            .route("/{album_id}", web::put().to(update_album))
            .route("/{album_id}", web::delete().to(delete_album))
            .route("/{album_id}/songs", web::get().to(get_album_songs))
    );
}