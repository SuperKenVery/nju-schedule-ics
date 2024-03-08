use axum::{
    routing::{get, post}, Router
};
use tokio::sync::Mutex;
use std::sync::Arc;

use super::{
    login,
    nojslogin,
    db::CookieDb,
    config::{parse_config, from_commandline}, error::AppError
};
use super::subscription::get_ical;
use super::db;

pub struct AppState {
    pub auth: Mutex<login::Authenticator>,
    pub cookie_db: Mutex<CookieDb>,
    pub site_url: String,       // e.g. http://localhost:8999   No trailing slash
}

pub async fn build_app(db: db::CookieDb, server_url: String) -> Result<Router,anyhow::Error> {
    let state=Arc::new(AppState{
        auth: Mutex::new(login::Authenticator::new(&db).await?),
        cookie_db: Mutex::new(db),
        site_url: server_url,
    });


    let app = Router::new()
        .route("/",                   get(nojslogin::redirect_to_nojs))
        // JSON API
        .route("/get_login_session",  get(login::new_login_session))
        .route("/login",              post(login::finish_login))
        // 0-js login
        .route("/nojs/index", get(nojslogin::get_index_html))
        .route("/nojs/captcha.png",   get(nojslogin::get_captcha_content))
        .route("/nojs/login",         post(nojslogin::login))
        .route("/nojs/style.css",get(nojslogin::get_style_css))
        .route("/:uuid/schedule.ics", get(get_ical))
        // .route("/test.ics",           get(test_ical))
        .with_state(state);

    Ok(app)
}

pub async fn start_server_from_config(filename: &str) -> Result<(),AppError> {
    let (app,config)=parse_config(filename).await?;

    let listener=tokio::net::TcpListener::bind(&config.listen_addr).await?;
    axum::serve(listener, app)
        .await?;

    Ok(())
}

pub async fn start_server_from_commandline() -> Result<(),AppError> {
    let (app,config)=from_commandline().await?;

    let listener=tokio::net::TcpListener::bind(&config.listen_addr).await?;
    axum::serve(listener, app)
        .await?;

    Ok(())
}

#[cfg(test)]
mod test{
    use super::*;
    use tokio;

    #[tokio::test]
    async fn start_server() {
        let db=db::CookieDb::new("sqlite://cookies.sqlite").await.unwrap();

        let app=build_app(db,"http://localhost:8999".into()).await.unwrap();

        println!("Starting server...");

        let listener=tokio::net::TcpListener::bind("0.0.0.0:8899").await.unwrap();
        axum::serve(listener, app)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn axum_works(){
        // build our application with a single route
        let app = Router::new().route("/", get(|| async { "Hello, World!" }));

        let listener=tokio::net::TcpListener::bind("0.0.0.0:8899").await.unwrap();
        axum::serve(listener, app)
            .await
            .unwrap();
    }

}
