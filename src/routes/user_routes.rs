use actix_web::web::{self};

use crate::handlers::user_handlers::{update_user, create_user};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("", web::post().to(create_user))
            .route("/{user_id}", web::patch().to(update_user))
    );
}