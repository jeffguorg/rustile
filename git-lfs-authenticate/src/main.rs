use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use serde_json::json;

use jsonwebtoken::{encode, EncodingKey, Header};

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    /// Optional name to operate on
    repo: String,

    #[clap(subcommand)]
    command: Commands,

    /// Sets a custom config file
    #[clap(
        short,
        long,
        parse(from_os_str),
        value_name = "FILE",
        default_value = "."
    )]
    config: PathBuf,

    /// Turn debugging information on
    #[clap(short, long, parse(from_occurrences))]
    debug: usize,
}

#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Download,
    Upload,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claim {
    sub: String,
    iat: chrono::DateTime<chrono::Utc>,

    command: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    // assert_eq!(args.repo.split("/").count(), 2);
    assert!(args.repo.ends_with(".git"));
    std::env::set_current_dir("/home/guochao")?;
    let secret = EncodingKey::from_secret("secret".as_ref());

    // if let Err(err) = dotenv::dotenv() {
    //     eprintln!("{}", err);
    // }

    // if let Err(err) = dotenv::from_path(args.config) {
    //     eprintln!("{}", err);
    // }

    // let conn = database::connection::from_env()?;
    // let users = database::user::query_users_by_public_key_fingerprint(
    //     &conn,
    //     std::env::var("FINGERPRINT")?,
    // )?;
    // for (idx, (user, pubkey)) in users.iter().enumerate() {
    //     eprintln!("{}: {} {}", idx, user.username, pubkey.fingerprint);
    // }
    // assert_eq!(users.len(), 1);

    let command = String::from(match args.command {
        Commands::Download => "download",
        Commands::Upload => "upload",
    });

    println!(
        "{}",
        json!({
            "header": {
                "Authorization": format!("Token {}", encode(&Header::default(), &Claim{
                    sub: String::from("git.jeffthecoder.xyz"),
                    iat: chrono::Utc::now(),

                    command,
                }, &secret)?),
            },
        })
        .to_string()
    );

    Ok(())
}
