use diesel::prelude::Insertable;
use diesel::{Queryable, Selectable};
use serde::{Serialize, Deserialize};

use crate::models::pagination_models::Pagination;

#[derive(Queryable, Insertable, Serialize, Deserialize, Selectable)]
#[diesel(table_name = crate::schema::albums)]
pub struct Album {
    pub id: String,
    pub name: String,
    pub artist_id: String,
    pub release_year: Option<i32>,
    pub cover_url: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::albums)]
pub struct NewAlbum {
    #[serde(skip_deserializing)]
    pub id: String,
    pub name: String,
    pub artist_id: String,
    pub release_year: Option<i32>,
    pub cover_url: Option<String>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::albums)]
pub struct UpdateAlbum {
    pub name: String,
    pub release_year: Option<i32>,
    pub cover_url: Option<String>,
}

#[derive(Deserialize)]
pub struct AlbumQuery {
    pub q: Option<String>,
    #[serde(flatten)]
    pub pagination: Pagination,
}