use actix_web::web::{self, scope};

use crate::handlers::session_handlers::{create_session, get_session, delete_session};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        scope("/sessions")
            .route("", web::post().to(create_session))
            .route("/{session_id}", web::get().to(get_session))
            .route("/{session_id}", web::delete().to(delete_session))
    );
}
