use anyhow::Result;
use clap::Parser;
use serde::Deserialize;
use toml;
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(author,version,about,long_about=None)]
struct Args {
    /// Config file filename
    #[arg(short, long)]
    config: String,
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub db_path: String,
    pub site_url: String,
}

const DEFAULT_CFG: &str = r#"
# The path to SQLite database
# which stores cookies
db_path="./cookies.sqlite"

# The URL this site is hosted
# No trailing slash
# Must start with https://
site_url="https://example.com/sub_dir"
"#;

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let config = std::fs::read_to_string(path);

        match config {
            Ok(config) => {
                let config: Config = toml::from_str(&config)?;

                Ok(config)
            }
            Err(e) => {
                error!("Failed to read config file: {}", e);
                if e.kind() == std::io::ErrorKind::NotFound {
                    info!("Creating default config file...");
                    let result = std::fs::write(path, DEFAULT_CFG.trim());
                    if let Err(e) = result {
                        error!("Failed to write default config file: {}", e)
                    }
                }

                Err(e.into())
            }
        }
    }

    pub fn from_cmdline() -> Result<Self> {
        let args = Args::parse();

        Self::from_file(&args.config)
    }

    pub fn from_default() -> Result<Self> {
        Self::from_file("./config.toml")
    }
}
