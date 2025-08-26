use diesel::{Queryable, Insertable, AsChangeset};
use chrono::NaiveDateTime;

#[derive(Queryable, Debug, serde::Serialize)]
pub struct Song {
    pub id: String,
    pub title: String,
    pub artist_id: String,
    pub album_id: Option<String>,
    pub genre: Option<String>,
    pub duration_seconds: i32,
    pub audio_url: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Insertable, serde::Deserialize)]
#[diesel(table_name = crate::schema::songs)]
pub struct NewSong {
    pub id: String,
    pub title: String,
    pub artist_id: String,
    pub album_id: Option<String>,
    pub genre: Option<String>,
    pub duration_seconds: i32,
    pub audio_url: String,
}

#[derive(AsChangeset, serde::Deserialize)]
#[diesel(table_name = crate::schema::songs)]
pub struct UpdateSong {
    pub title: Option<String>,
    pub artist_id: Option<String>,
    pub album_id: Option<String>,
    pub genre: Option<String>,
    pub duration_seconds: Option<i32>,
    pub audio_url: Option<String>,
}

#[derive(serde::Serialize)]
pub struct SongResponse {
    pub id: String,
    pub title: String,
    pub artist_id: String,
    pub album_id: Option<String>,
    pub genre: Option<String>,
    pub duration_seconds: i32,
    pub audio_url: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl From<Song> for SongResponse {
    fn from(s: Song) -> Self {
        SongResponse {
            id: s.id,
            title: s.title,
            artist_id: s.artist_id,
            album_id: s.album_id,
            genre: s.genre,
            duration_seconds: s.duration_seconds,
            audio_url: s.audio_url,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}
