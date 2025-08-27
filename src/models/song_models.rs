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
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub sftp_path: String,
}

#[derive(Insertable, serde::Deserialize)]
#[diesel(table_name = crate::schema::songs)]
pub struct NewSong {
    #[serde(skip_deserializing)]
    pub id: String,
    pub title: String,
    pub artist_id: String,
    pub album_id: Option<String>,
    pub genre: Option<String>,
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
    pub genre: Option<String>,
    pub duration_seconds: Option<i32>,
    pub sftp_path: Option<String>,
}

#[derive(serde::Serialize)]
pub struct SongResponse {
    pub id: String,
    pub title: String,
    pub artist_id: String,
    pub album_id: Option<String>,
    pub genre: Option<String>,
    pub duration_seconds: i32,
    pub sftp_path: String,
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
            sftp_path: s.sftp_path,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}
