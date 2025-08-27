use diesel::prelude::{AsChangeset, Associations, Identifiable, Insertable, Queryable};
use chrono::{NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --------------------- Playlist Models ---------------------
#[derive(Queryable, Identifiable, Associations, Serialize)]
#[diesel(table_name = crate::schema::playlists)]
#[diesel(belongs_to(crate::models::user_models::UserResponse, foreign_key = user_id))]
pub struct Playlist {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_public: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::playlists)]
pub struct NewPlaylist {
    #[serde(skip_deserializing)]
    pub id: String,
    #[serde(skip_deserializing)]
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_public: Option<bool>,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::playlists)]
pub struct UpdatePlaylist {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,
}

// --------------------- Playlist Songs Models ---------------------
#[derive(Queryable, Identifiable, Associations, Serialize)]
#[diesel(table_name = crate::schema::playlist_songs)]
#[diesel(primary_key(playlist_id, song_id))]
#[diesel(belongs_to(crate::models::playlist_models::Playlist, foreign_key = playlist_id))]
#[diesel(belongs_to(crate::models::song_models::Song, foreign_key = song_id))]
pub struct PlaylistSong {
    pub playlist_id: String,
    pub song_id: String,
    pub position: i32,
    pub added_at: Option<NaiveDateTime>,
}

#[derive(Deserialize)]
pub struct AddSongRequest {
    pub song_id: String,
    pub position: i32,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::playlist_songs)]
pub struct NewPlaylistSong {
    pub playlist_id: String,
    pub song_id: String,
    pub position: i32,
}

#[allow(dead_code)]
// --------------------- Response Models ---------------------
#[derive(Serialize)]
pub struct PlaylistResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl From<Playlist> for PlaylistResponse {
    fn from(p: Playlist) -> Self {
        Self {
            id: Uuid::parse_str(&p.id).unwrap_or(Uuid::nil()),
            user_id: Uuid::parse_str(&p.user_id).unwrap_or(Uuid::nil()),
            name: p.name,
            description: p.description,
            is_public: p.is_public.unwrap_or(false),
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

#[allow(dead_code)]
#[derive(Serialize)]
pub struct PlaylistSongsResponse {
    pub playlist_id: Uuid,
    pub songs: Vec<crate::models::song_models::SongResponse>,
}
