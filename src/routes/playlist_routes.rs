use actix_web::web;

use crate::handlers::playlist_handlers::{
    list_playlists, create_playlist, get_playlist, update_playlist, delete_playlist,
    list_playlist_songs, add_song_to_playlist, remove_song_from_playlist
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    // Playlists
    cfg.service(
        web::scope("/users/{user_id}/playlists")
            .route("", web::get().to(list_playlists))
            .route("", web::post().to(create_playlist))
            .route("/{playlist_id}", web::get().to(get_playlist))
            .route("/{playlist_id}", web::put().to(update_playlist))
            .route("/{playlist_id}", web::delete().to(delete_playlist))
    );

    // Songs in Playlists
    cfg.service(
        web::scope("/users/{user_id}/playlists/{playlist_id}/songs")
            .route("", web::get().to(list_playlist_songs))
            .route("", web::post().to(add_song_to_playlist))
            .route("/{song_id}", web::delete().to(remove_song_from_playlist))
    );
}
