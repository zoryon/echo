use diesel::prelude::{Insertable, Queryable};
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Queryable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::sessions)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub created_at: Option<NaiveDateTime>,
    pub expires_at: Option<NaiveDateTime>,
}

#[derive(Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::sessions)]
pub struct NewSession {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub created_at: Option<NaiveDateTime>,
    pub expires_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub token: String,
    pub expires_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSession {
    pub user_id: String,
    pub password: String,
}