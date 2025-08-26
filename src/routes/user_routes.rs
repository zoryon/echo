use actix_web::web::{self};

use crate::handlers::user_handlers::update_user;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
                    .route("/{user_id}", web::patch().to(update_user))
    );
}