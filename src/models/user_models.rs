use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::{prelude::Queryable, AsChangeset, Selectable, Insertable};

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::users)]
#[diesel(treat_none_as_null = false)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub avatar_url: Option<String>,
}

#[allow(dead_code)]
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub avatar_url: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub is_admin: bool,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub avatar_url: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

// Payload for creating a new user
#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
    pub avatar_url: Option<String>,
    pub is_admin: Option<bool>,
}

// Insertable model for users table
#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub avatar_url: Option<String>,
    pub is_admin: bool,
}
