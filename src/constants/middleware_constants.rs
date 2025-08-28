use actix_web::http::Method;
use std::collections::HashSet;
use once_cell::sync::Lazy;

// Define all admin-only routes (path + method) as a static set
pub static ADMIN_ONLY_ROUTES: Lazy<HashSet<(&'static str, Method)>> = Lazy::new(|| {
    let mut set = HashSet::new();

    // Existing admin-only route example
    set.insert(("/api/users", Method::POST));

    // New admin-only routes for songs
    set.insert(("/api/songs", Method::POST));
    set.insert(("/api/songs/{song_id}", Method::PUT));
    set.insert(("/api/songs/{song_id}", Method::DELETE));

    set
});
