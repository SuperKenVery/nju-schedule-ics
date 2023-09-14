use axum::{
    routing::{get, post}, Router,
};
use tokio::sync::Mutex;
use std::sync::Arc;

use super::{
    login,
    nojslogin,
    db::RedisDb,
    config::{parse_config, from_commandline}, error::AppError
};
use super::subscription::{get_ical,test_ical};
use super::db;

pub struct AppState {
    pub auth: Mutex<login::Authenticator>,
    pub cookie_db: Mutex<RedisDb>,
    pub site_url: String,       // e.g. http://localhost:8999   No trailing slash
}

pub async fn build_app(db: db::RedisDb, server_url: String) -> Result<Router,anyhow::Error> {
    let state=Arc::new(AppState{
        auth: Mutex::new(login::Authenticator::new()),
        cookie_db: Mutex::new(db),
        site_url: server_url
    });


    let app = Router::new()
        .route("/",                   get(nojslogin::get_index_html))
        // JSON API
        .route("/get_login_session",  get(login::new_login_session))
        .route("/login",              post(login::finish_login))
        // 0-js login
        .route("/nojs/captcha.png",   get(nojslogin::get_captcha_content))
        .route("/nojs/login",         post(nojslogin::login))
        .route("/:uuid/schedule.ics", get(get_ical))
        .route("/test.ics",           get(test_ical))
        .with_state(state);

    Ok(app)
}

pub async fn start_server_from_config(filename: &str) -> Result<(),AppError> {
    let (app,config)=parse_config(filename).await?;

    axum::Server::bind(&config.listen_addr.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

pub async fn start_server_from_commandline() -> Result<(),AppError> {
    let (app,config)=from_commandline().await?;

    axum::Server::bind(&config.listen_addr.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[cfg(test)]
mod test{
    use super::*;
    use tokio;

    #[tokio::test]
    async fn start_server() {
        println!("Connecting to redis...");
        let url="redis://127.0.0.1:6379/";
        let db=db::RedisDb::new(url).await.unwrap();

        let app=build_app(db,"http://192.168.1.179:8999".into()).await.unwrap();

        println!("Starting server...");
        axum::Server::bind(&"0.0.0.0:8999".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn axum_works(){
        // build our application with a single route
        let app = Router::new().route("/", get(|| async { "Hello, World!" }));

        // run it with hyper on localhost:3000
        axum::Server::bind(&"0.0.0.0:8899".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}
