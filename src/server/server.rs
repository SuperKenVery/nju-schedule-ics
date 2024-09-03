use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::db;
use super::subscription::get_ical;
use super::{
    config::{from_commandline, parse_config},
    db::CookieDb,
    error::AppError,
    login, nojslogin,
};

pub struct AppState {
    pub cookie_db: Arc<Mutex<CookieDb>>,
    pub site_url: String,
}

pub async fn build_app(
    db: Arc<Mutex<db::CookieDb>>,
    site_url: String,
) -> Result<Router, anyhow::Error> {
    let state = Arc::new(AppState {
        cookie_db: db.clone(),
        site_url,
    });

    let app = Router::new()
        .route("/", get(nojslogin::redirect_to_nojs))
        // JSON API
        .route("/get_login_session", get(login::new_login_session))
        .route("/login", post(login::finish_login))
        // 0-js login
        .route("/nojs/index", get(nojslogin::get_index_html))
        .route("/nojs/captcha.png", get(nojslogin::get_captcha_content))
        .route("/nojs/login", post(nojslogin::login))
        .route("/nojs/style.css", get(nojslogin::get_style_css))
        .route("/:uuid/schedule.ics", get(get_ical))
        // .route("/test.ics",           get(test_ical))
        .with_state(state);

    Ok(app)
}

pub async fn start_server_from_config(filename: &str) -> Result<(), AppError> {
    let (app, config) = parse_config(filename).await?;

    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

pub async fn start_server_from_commandline() -> Result<(), AppError> {
    let (app, config) = from_commandline().await?;

    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn start_server() {
        let db = db::CookieDb::new("sqlite://cookies.sqlite").await.unwrap();
        let db = Arc::new(Mutex::new(db));

        let app = build_app(db, "https://localhost:8899".to_string())
            .await
            .unwrap();

        println!("Starting server...");

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
