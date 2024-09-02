use clap::Parser;
use axum::Router;
use serde::Deserialize;
use super::error::AppError;
use super::server::build_app;
use toml;
use super::db;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Parser,Debug)]
#[command(author,version,about,long_about=None)]
struct Args{
    // Config file filename (with path)
    #[arg(short,long)]
    config: String,
}

#[derive(Deserialize)]
pub struct Config{
    pub db_path: String,
    pub listen_addr: String,
    pub site_url: String,
}

pub async fn parse_config(path: &str) -> Result<(Router,Config),AppError> {
    let config=std::fs::read_to_string(path);

    match config {
        Ok(config) => {
            let config: Config=toml::from_str(&config)?;

            let db=db::CookieDb::new(&config.db_path).await?;
            let db=Arc::new(Mutex::new(db));

            Ok(
                (
                    build_app(db, config.site_url.clone()).await?,
                    config,
                )
            )
        },
        Err(e) => {
            println!("Failed to read config file: {}",e);
            if e.kind()==std::io::ErrorKind::NotFound {
                println!("Creating default config file...");
                let result=std::fs::write(path, r#"
# The path to SQLite database
# which stores cookies
db_path="./cookies.sqlite"

# The URL this site is hosted
# No trailing slash
# Must start with https://
site_url="https://example.com/sub_dir"

# Listen address&port
# This is different from site_url, as you'll probably
# use a reverse proxy in front of this.
listen_addr="0.0.0.0:8899"
"#.trim());
                if let Err(e)=result{
                    println!("Failed to write default config file: {}",e)
                }
            }

            Err(e.into())
        }
    }

}

pub async fn from_commandline() -> Result<(Router,Config),AppError> {
    let args=Args::parse();

    parse_config(&args.config).await
}
