use actix_web::{middleware::Logger, App, HttpServer};
use middleware::token_extractor::JWTSecret;
use s3::{creds::Credentials, Bucket, Region};

use handlers::*;

pub mod handlers;
pub mod middleware;
pub mod templates;

#[derive(Debug, Clone)]
pub struct AppContext {
    pub bucket: Bucket,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_current_dir(std::env::var("HOME").unwrap_or(String::from("/")))?;

    env_logger::init();

    HttpServer::new(|| {
        let mut bucket = Bucket::new_with_path_style(
            std::env::var("AWS_BUCKET_NAME").unwrap().as_str(),
            Region::Custom {
                region: std::env::var("AWS_ENDPOINT_REGION").unwrap(),
                endpoint: std::env::var("AWS_ENDPOINT_PREFIX").unwrap(),
            },
            Credentials::from_env().unwrap(),
        )
        .unwrap();
        bucket.set_subdomain_style();

        App::new()
            .app_data(JWTSecret(Box::new(
                std::env::var("SECRET").unwrap().as_bytes().to_vec(),
            )))
            .data(AppContext { bucket })
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(lfs_lock_verify)
            .service(lfs_objects_batch)
            .service(git_repo_detail)
            .service(git_repo)
            .service(index)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
