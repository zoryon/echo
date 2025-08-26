use actix_web::web::{self, scope};

use crate::handlers::session_handlers::{create_session, get_current_session, delete_current_session};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        scope("/sessions")
            .route("", web::post().to(create_session)) // POST /api/sessions
            .route("/current", web::get().to(get_current_session)) // GET /api/sessions/current
            .route("/current", web::delete().to(delete_current_session)) // DELETE /api/sessions/current
    );
}
