pub mod playlist_routes;
pub mod favorite_routes;
pub mod session_routes;
pub mod user_routes;
pub mod song_routes;
pub mod album_routes;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    playlist_routes::configure(cfg);
    favorite_routes::configure(cfg);
    session_routes::configure(cfg);
    user_routes::configure(cfg);
    song_routes::configure(cfg);
    album_routes::configure(cfg);
}