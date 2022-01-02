#[macro_use]
extern crate diesel;

pub mod schema;
pub mod models;

pub mod user;

pub mod connection {
    use diesel::{MysqlConnection, Connection};

    pub fn from_env() -> Result<MysqlConnection, diesel::ConnectionError> {
         MysqlConnection::establish(std::env::var("DATABASE_URL").unwrap_or(String::from("mysql://root:123456@localhost/git")).as_str())
    }
}
