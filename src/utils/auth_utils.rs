use actix_web::{HttpResponse, web::ReqData};

use crate::models::token_models::Claims;

/// Check that the requested resource belongs to the logged-in user.
/// Returns `Ok(&str)` with the user_id if authorized, otherwise returns a 404 response.
pub fn check_ownership<'a>(
    path_user_id: &'a str,
    claims: &'a ReqData<Claims>,
) -> Result<&'a str, HttpResponse> {
    let logged_in_user_id: &String = &claims.sub;
    if logged_in_user_id != path_user_id {
        // 404 Not Found to avoid leaking info about other users
        Err(HttpResponse::NotFound().body("Not Found"))
    } else {
        Ok(logged_in_user_id)
    }
}
