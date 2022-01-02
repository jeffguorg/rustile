use super::schema::{user, public_key};

#[derive(Queryable)]
pub struct User {
    pub uuid: String,
    pub username: String,

    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub deleted_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[table_name="user"]
pub struct NewUser {
    pub uuid: String,
    pub username: String,
}

#[derive(Queryable)]
pub struct PublicKey {
    pub fingerprint: String,
    pub user: String,
}

#[derive(Insertable)]
#[table_name="public_key"]
pub struct NewPublicKey {
    pub fingerprint: String,
    pub user: String,
}