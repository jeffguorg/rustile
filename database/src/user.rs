use crate::models::{NewUser, PublicKey, User};
use crate::schema::{public_key, user};
use diesel::mysql::{MysqlConnection};
use diesel::prelude::*;
// use diesel::RunQueryDsl;
// use diesel::*;

pub fn create_user(
    conn: &MysqlConnection,
    username: String,
) -> Result<usize, diesel::result::Error> {
    let new_user = &NewUser {
        uuid: uuid::Uuid::new_v4().to_string(),
        username: username,
    };
    diesel::insert_into(user::table)
        .values(new_user)
        .execute(conn)
}

pub fn query_users_by_id(
    conn: &MysqlConnection,
    uuid: String,
) -> Result<Vec<User>, diesel::result::Error> {
    user::dsl::user
        .filter(user::dsl::uuid.eq(uuid))
        .load::<User>(conn)
}

pub fn query_users_by_public_key_fingerprint(conn: &MysqlConnection, fingerprint: String) -> Result<Vec<(User, PublicKey)>, diesel::result::Error> {
    user::dsl::user
        .inner_join(public_key::table)
        .filter(public_key::dsl::fingerprint.eq(fingerprint)).load::<(User, PublicKey)>(conn)
}
