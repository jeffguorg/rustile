use actix_web::{App, HttpServer};
use handlers::*;

pub mod handlers;
pub mod templates;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_current_dir(std::env::var("HOME").unwrap_or(String::from("/")))?;
    HttpServer::new(|| {
        App::new()
            .service(git_repo_detail)
            .service(git_repo)
            .service(index)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
