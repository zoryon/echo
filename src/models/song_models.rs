use diesel::{prelude::QueryableByName, AsChangeset, Insertable, Queryable};
use chrono::NaiveDateTime;
use serde::Serialize;
use diesel::sql_types::{Text, Integer, Nullable};

#[derive(Queryable, Debug, serde::Serialize)]
pub struct Song {
    pub id: String,
    pub title: String,
    pub artist_id: String,
    pub album_id: Option<String>,
    pub genre_id: Option<i32>,
    pub duration_seconds: i32,
    pub sftp_path: String,
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
    pub sftp_path: String,
}

#[derive(AsChangeset, serde::Deserialize)]
#[diesel(table_name = crate::schema::songs)]
pub struct UpdateSong {
    pub title: Option<String>,
    pub artist_id: Option<String>,
    pub album_id: Option<String>,
    pub genre_id: Option<i32>,
    pub duration_seconds: Option<i32>,
    pub sftp_path: Option<String>,
}

#[derive(QueryableByName, Serialize)]
pub struct SongResponse {
    #[sql_type = "Text"]
    id: String,
    #[sql_type = "Text"]
    title: String,
    
    #[sql_type = "Text"]
    artist_id: String,
    #[sql_type = "Text"]
    artist_name: String,
    
    #[sql_type = "Text"]
    album_id: String,
    #[sql_type = "Text"]
    album_name: String,
    
    #[sql_type = "Text"]
    genre_id: String,
    #[sql_type = "Text"]
    genre_name: String,
    
    #[sql_type = "Integer"]
    duration_seconds: i32,
    
    #[sql_type = "Text"]
    sftp_path: String,
    
    #[sql_type = "Nullable<Text>"]
    created_at: Option<String>,
    #[sql_type = "Nullable<Text>"]
    updated_at: Option<String>,
}

impl From<Song> for SongResponse {
    fn from(s: Song) -> Self {
        SongResponse {
            id: s.id,
            title: s.title,
            
            artist_id: s.artist_id.clone(),
            artist_name: String::new(),
            
            album_id: s.album_id.clone().unwrap_or_default(),
            album_name: String::new(), 
            
            genre_id: s.genre_id.map(|g| g.to_string()).unwrap_or_default(),
            genre_name: String::new(), 
            
            duration_seconds: s.duration_seconds,
            sftp_path: s.sftp_path,
            
            created_at: s.created_at.map(|dt| dt.to_string()),
            updated_at: s.updated_at.map(|dt| dt.to_string()),
        }
    }
}
