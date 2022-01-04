use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
    let warning_not_configured = include_bytes!("warnings/ssh_not_configured.txt");

    let args = Cli::parse();

    std::env::set_current_dir("/home/guochao")?;
    assert!(args.repo.ends_with(".git"));
    assert!(Path::new(&args.repo).exists());

    if let Err(_) = std::env::var("SSH_KEY_FINGERPRINT") {
        println!("{}", String::from_utf8(warning_not_configured.to_vec())?);
        panic!("no proper environment.");
    }

    let secret = EncodingKey::from_secret(std::env::var("SECRET").unwrap().as_bytes());

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
