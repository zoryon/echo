use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::AsChangeset;

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::users)]
#[diesel(treat_none_as_null = false)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub avatar_url: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
