// @generated automatically by Diesel CLI.

diesel::table! {
    albums (id) {
        #[max_length = 36]
        id -> Char,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 36]
        artist_id -> Char,
        release_year -> Nullable<Integer>,
        cover_url -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    artists (id) {
        #[max_length = 36]
        id -> Char,
        #[max_length = 100]
        name -> Varchar,
        bio -> Nullable<Text>,
        image_url -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    favorites (user_id, song_id) {
        #[max_length = 36]
        user_id -> Char,
        #[max_length = 36]
        song_id -> Char,
        added_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    playlist_songs (playlist_id, song_id) {
        #[max_length = 36]
        playlist_id -> Char,
        #[max_length = 36]
        song_id -> Char,
        position -> Integer,
        added_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    playlists (id) {
        #[max_length = 36]
        id -> Char,
        #[max_length = 36]
        user_id -> Char,
        #[max_length = 100]
        name -> Varchar,
        description -> Nullable<Text>,
        is_public -> Nullable<Bool>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    sessions (id) {
        #[max_length = 36]
        id -> Char,
        #[max_length = 36]
        user_id -> Char,
        #[max_length = 255]
        token -> Varchar,
        created_at -> Nullable<Timestamp>,
        expires_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    songs (id) {
        #[max_length = 36]
        id -> Char,
        #[max_length = 200]
        title -> Varchar,
        #[max_length = 36]
        artist_id -> Char,
        #[max_length = 36]
        album_id -> Nullable<Char>,
        #[max_length = 50]
        genre -> Nullable<Varchar>,
        duration_seconds -> Integer,
        audio_url -> Text,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        #[max_length = 36]
        id -> Char,
        #[max_length = 50]
        username -> Varchar,
        password_hash -> Text,
        avatar_url -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        is_admin -> Bool,
    }
}

diesel::joinable!(albums -> artists (artist_id));
diesel::joinable!(favorites -> songs (song_id));
diesel::joinable!(favorites -> users (user_id));
diesel::joinable!(playlist_songs -> playlists (playlist_id));
diesel::joinable!(playlist_songs -> songs (song_id));
diesel::joinable!(playlists -> users (user_id));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(songs -> albums (album_id));
diesel::joinable!(songs -> artists (artist_id));

diesel::allow_tables_to_appear_in_same_query!(
    albums,
    artists,
    favorites,
    playlist_songs,
    playlists,
    sessions,
    songs,
    users,
);
