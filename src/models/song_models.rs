use diesel::{prelude::QueryableByName, AsChangeset, Insertable, Queryable};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use diesel::sql_types::{Text, Integer, Nullable, Timestamp};

#[derive(Deserialize)]
pub struct SongQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub name: Option<String>,
    pub genre: Option<String>,
    pub artist: Option<String>,
    pub sort: Option<String>, // e.g. "release_date" or "-release_date"
    pub random: Option<bool>,
}

#[derive(Queryable, Debug, serde::Serialize)]
pub struct Song {
    pub id: String,
    pub title: String,
    pub artist_id: String,
    pub album_id: Option<String>,
    pub genre_id: Option<i32>,
    pub duration_seconds: i32,
    pub object_url: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Insertable, serde::Deserialize)]
#[diesel(table_name = crate::schema::songs)]
pub struct NewSong {
    #[serde(skip_deserializing)]
    pub id: String,
    pub title: String,
    pub artist_id: String,
    pub album_id: Option<String>,
    pub genre_id: Option<i32>,
    pub duration_seconds: i32,
    #[serde(skip_deserializing)]
    pub object_url: String,
}

#[derive(AsChangeset, serde::Deserialize)]
#[diesel(table_name = crate::schema::songs)]
pub struct UpdateSong {
    pub title: Option<String>,
    pub artist_id: Option<String>,
    pub album_id: Option<String>,
    pub genre_id: Option<i32>,
    pub duration_seconds: Option<i32>,
}


#[derive(Clone, QueryableByName, Serialize)]
pub struct SongResponse {
    #[diesel(sql_type = Text)]
    pub id: String,
    #[diesel(sql_type = Text)]
    pub title: String,

    #[diesel(sql_type = Text)]
    pub artist_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub artist_name: Option<String>,

    #[diesel(sql_type = Nullable<Text>)]
    pub album_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub album_name: Option<String>,

    #[diesel(sql_type = Nullable<Integer>)]
    pub genre_id: Option<i32>,
    #[diesel(sql_type = Nullable<Text>)]
    pub genre_name: Option<String>,

    #[diesel(sql_type = Integer)]
    pub duration_seconds: i32,

    #[diesel(sql_type = Text)]
    pub object_url: String,

    #[diesel(sql_type = Nullable<Timestamp>)]
    pub created_at: Option<NaiveDateTime>,
    #[diesel(sql_type = Nullable<Timestamp>)]
    pub updated_at: Option<NaiveDateTime>,
}
impl From<Song> for SongResponse {
    fn from(s: Song) -> Self {
        SongResponse {
            id: s.id,
            title: s.title,
            
            artist_id: s.artist_id.clone(),
            artist_name: None, 

            album_id: s.album_id.clone(), 
            album_name: None, 
            
            genre_id: s.genre_id,
            genre_name: None, 
            
            duration_seconds: s.duration_seconds,
            object_url: s.object_url,
            
            created_at: s.created_at,   
            updated_at: s.updated_at, 
        }
    }
}
