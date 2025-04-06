use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub id: String,
    pub username: String,
    pub password: String,
}

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct Note {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub content: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::notes)]
pub struct NewNote {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub content: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::notes)]
pub struct UpdateNote {
    pub title: Option<String>,
    pub content: Option<String>,
    #[diesel(column_name = updated_at)]
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Deserialize)]
pub struct AuthData {
    pub username: String,
    pub password: String,
}