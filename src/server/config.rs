use anyhow::{Result, ensure};
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
    /// OTLP endpoint, e.g. https://otlp-gateway-prod-ap-southeast-1.grafana.net/otlp
    pub otel_endpoint: Option<String>,
    /// Grafana Cloud instance ID (the "Username" shown in the OTLP credentials page)
    pub otel_instance_id: Option<String>,
    /// Grafana Cloud API token (the "Password" shown in the OTLP credentials page)
    pub otel_token: Option<String>,
    /// Global rate limit for client login requests.
    #[serde(default)]
    pub login_rate_limit: LoginRateLimitConfig,
}

#[derive(Deserialize, Clone)]
pub struct LoginRateLimitConfig {
    /// Number of new login tokens generated each second.
    pub tokens_per_second: f64,
    /// Maximum number of accumulated login tokens.
    pub capacity: u32,
}

impl Default for LoginRateLimitConfig {
    fn default() -> Self {
        Self {
            tokens_per_second: 0.2,
            capacity: 5,
        }
    }
}

impl LoginRateLimitConfig {
    pub(crate) fn validate(&self) -> Result<()> {
        ensure!(
            self.tokens_per_second.is_finite() && self.tokens_per_second > 0.0,
            "login_rate_limit.tokens_per_second must be a finite number greater than zero"
        );
        ensure!(
            self.capacity > 0,
            "login_rate_limit.capacity must be greater than zero"
        );
        Ok(())
    }
}

const DEFAULT_CFG: &str = r#"
# The path to SQLite database
# which stores cookies
db_path="./cookies.sqlite"

# The URL this site is hosted
# No trailing slash
# Must start with https://
site_url="https://example.com/sub_dir"

# Global token bucket for client login requests.
# This example allows a burst of 5 logins, then replenishes one token every 5 seconds.
[login_rate_limit]
tokens_per_second=0.2
capacity=5
"#;

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let config = std::fs::read_to_string(path);

        match config {
            Ok(config) => {
                let config: Config = toml::from_str(&config)?;
                config.login_rate_limit.validate()?;

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

#[cfg(test)]
mod tests {
    use super::{Config, DEFAULT_CFG};

    #[test]
    fn default_config_is_valid() {
        let config: Config = toml::from_str(DEFAULT_CFG).unwrap();
        config.login_rate_limit.validate().unwrap();
    }

    #[test]
    fn legacy_config_gets_default_login_rate_limit() {
        let config: Config = toml::from_str(
            r#"
                db_path = "./cookies.sqlite"
                site_url = "https://example.com"
            "#,
        )
        .unwrap();

        assert_eq!(config.login_rate_limit.capacity, 5);
        assert_eq!(config.login_rate_limit.tokens_per_second, 0.2);
    }
}
