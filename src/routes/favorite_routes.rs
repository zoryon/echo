use actix_web::web;

use crate::handlers::favorite_handlers::{
    list_favorites, add_favorite, remove_favorite
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users/{user_id}/favorites/songs")
            .route("", web::get().to(list_favorites))
            .route("", web::post().to(add_favorite))
            .route("/{song_id}", web::delete().to(remove_favorite))
    );
}
