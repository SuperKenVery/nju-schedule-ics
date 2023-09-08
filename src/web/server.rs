use axum::{
    routing::{get, post},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use std::{collections::HashMap, cell::RefCell};
use uuid::Uuid;
use std::error::Error;
use std::sync::Arc;

use crate::nju::login::{LoginCredential,LoginOperation};
use super::login::{Authenticator, new_login_session,finish_login};
use super::subscription::{get_ical,test_ical};

pub struct AppState {
    pub auth: Mutex<Authenticator>,
    // TODO: cookie db should be persistent!
    pub cookie_db: Mutex<HashMap<String, LoginCredential>>,
}

pub fn build_app() -> Router {
    let state=Arc::new(AppState{
        auth: Mutex::new(Authenticator::new()),
        cookie_db: Mutex::new(HashMap::new()),
    });


    let app = Router::new()
        .route("/",                     get(|| async { "Hello, World!" }))
        .route("/get_login_session",    get(new_login_session))
        .route("/login",                post(finish_login))
        .route("/:uuid/schedule.ics",   get(get_ical))
        .route("/test.ics",             get(test_ical))
        .with_state(state);

    app
}

#[cfg(test)]
mod test{
    use super::*;
    use tokio;

    #[tokio::test]
    async fn start_server() {
        let app=build_app();

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
