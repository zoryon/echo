use diesel::prelude::{AsChangeset, Insertable, Queryable};
use serde::{Serialize, Deserialize};

use crate::schema::genres;

#[allow(dead_code)]
#[derive(Queryable, Serialize, Debug)]
pub struct Genre {
    pub id: i32,
    pub name: String,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = genres)]
pub struct NewGenre {
    pub name: String,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::genres)]
pub struct UpdateGenre {
    pub name: Option<String>,
}
