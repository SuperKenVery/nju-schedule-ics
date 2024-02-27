use clap::Parser;
use axum::Router;
use serde::Deserialize;
use super::error::AppError;
use super::server::build_app;
use toml;
use super::db;

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
    pub site_url: String,
    pub listen_addr: String,
}

pub async fn parse_config(path: &str) -> Result<(Router,Config),AppError> {
    let config=std::fs::read_to_string(path);

    match config {
        Ok(config) => {
            let config: Config=toml::from_str(&config)?;

            let db=db::CookieDb::new(&config.db_path).await?;

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

# The base URL of this site
# Don't add the trailing slash
site_url="https://example.com/example/sub/directory"

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
