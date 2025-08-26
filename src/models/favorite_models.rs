use diesel::prelude::{Insertable, Queryable};
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};

#[allow(dead_code)]
#[derive(Queryable, Serialize)]
pub struct Favorite {
    pub user_id: String,
    pub song_id: String,
    pub added_at: Option<NaiveDateTime>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::favorites)]
pub struct NewFavorite {
    pub user_id: String,
    pub song_id: String,
}
