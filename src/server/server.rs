use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use colog;
use colog::format::CologStyle;
use colored::Colorize;
use env_logger::fmt::Formatter;
use log::{Level, Record};
use std::io::Write;
use std::{result, sync::Arc};
use tokio::sync::Mutex;

use super::db;
use super::subscription::get_ical;
use super::{
    config::{from_commandline, parse_config},
    db::CookieDb,
    error::AppError,
    login, nojslogin,
};
use crate::schedule::holidays::HolidayCal;

pub struct AppState {
    pub cookie_db: Arc<Mutex<CookieDb>>,
    pub site_url: String,
    pub hcal: HolidayCal,
}

pub async fn build_app(db: Arc<Mutex<db::CookieDb>>, site_url: String) -> Result<Router> {
    let state = Arc::new(AppState {
        cookie_db: db.clone(),
        site_url,
        hcal: HolidayCal::from_shuyz().await?,
    });

    let app = Router::new()
        .route("/", get(nojslogin::redirect_to_nojs))

        .route("/nojs/index", get(nojslogin::get_index_html))
        .route("/nojs/captcha.png", get(nojslogin::get_captcha_content))
        .route("/nojs/login", post(nojslogin::login))
        .route("/nojs/style.css", get(nojslogin::get_style_css))
        .route("/:uuid/schedule.ics", get(get_ical))
        // .route("/test.ics",           get(test_ical))
        .with_state(state);

    Ok(app)
}

fn init_log() {
    colog::default_builder()
        .format(colog::formatter(LogTimePrefix))
        .filter(Some("nju_schedule_ics"), log::LevelFilter::Debug)
        .init();
}

pub async fn start_server_from_config(filename: &str) -> Result<(), AppError> {
    init_log();
    let (app, config) = parse_config(filename).await?;

    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

pub async fn start_server_from_commandline() -> Result<(), AppError> {
    init_log();
    let (app, config) = from_commandline().await?;

    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

pub struct LogTimePrefix;

impl CologStyle for LogTimePrefix {
    fn prefix_token(&self, level: &Level) -> String {
        format!(
            "[{}] {}",
            chrono::Local::now()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
                .blue()
                .bold(),
            self.level_color(level, self.level_token(level))
        )
    }

    // Show module name in log
    // fn format(&self, buf: &mut Formatter, record: &Record<'_>) -> Result<(), std::io::Error> {
    //     write!(
    //         buf,
    //         "{} [{}] {} - {}\n",
    //         self.prefix_token(&record.level()),
    //         record.module_path().unwrap_or("unknown"),
    //         record.level(),
    //         record.args()
    //     )
    // }
}

#[cfg(test)]
mod test {
    use super::*;
    use log::info;
    use tokio;

    #[tokio::test]
    async fn start_server() {
        let db = db::CookieDb::new("sqlite://cookies_local.sqlite")
            .await
            .unwrap();
        let db = Arc::new(Mutex::new(db));

        let app = build_app(db, "https://localhost:8899".to_string())
            .await
            .unwrap();

        info!("Starting server...");

        let listener = tokio::net::TcpListener::bind("0.0.0.0:8899").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }

    #[tokio::test]
    async fn axum_works() {
        // build our application with a single route
        let app = Router::new().route("/", get(|| async { "Hello, World!" }));

        let listener = tokio::net::TcpListener::bind("0.0.0.0:8899").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }
}
