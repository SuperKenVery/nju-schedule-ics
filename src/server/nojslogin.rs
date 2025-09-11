//! HTTP handlers for the no-js login

use super::error::AppError;
use super::server::AppState;

use anyhow::Result;
use axum::{
    extract::{Form, State},
    http::{header, status::StatusCode, HeaderMap},
    response::IntoResponse,
};
use axum_macros::debug_handler;
use serde::Deserialize;
use std::sync::Arc;

#[debug_handler]
pub async fn get_captcha_content(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let mut db = state.cookie_db.lock().await;
    let session = db.new_session().await?;
    let captcha = db.get_session_captcha(&session).await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        format!("SESSION={}; SameSite=Strict; HttpOnly;", session).try_into()?,
    );

    Ok((headers, captcha.clone()))
}

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
    captcha_answer: String,
    school_api: String,
}

#[debug_handler]
pub async fn login(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Form(LoginForm {
        username,
        password,
        captcha_answer,
        school_api,
    }): Form<LoginForm>,
) -> Result<impl IntoResponse, AppError> {
    let cookies = headers
        .get("cookie")
        .ok_or("No cookie")
        .map_err(anyhow::Error::msg)?
        .to_str()?;
    let cookies = cookies
        .split(";")
        .map(|s| {
            let (k, v) = s
                .split_once("=")
                .ok_or(format!("Invalid cookie: {}", s))
                .map_err(anyhow::Error::msg)?;
            Ok((k, v))
        })
        .collect::<Result<Vec<(&str, &str)>>>()?
        .into_iter();
    let session = cookies
        .filter(|(k, _v)| k == &"SESSION")
        .collect::<Vec<(&str, &str)>>()
        .get(0)
        .ok_or("No SESSION cookie")
        .map_err(anyhow::Error::msg)?
        .1;

    let mut db = state.cookie_db.lock().await;
    db.session_login(session, &username, &password, &captcha_answer)
        .await?;

    let subscription_html = include_str!("../html/subscription.html");
    let subscription_html = subscription_html.replace(
        "SUBSCRIPTION_LINK",
        format!(
            "{}/{}/schedule.ics",
            state.site_url.replace("https://", "webcal://"),
            session
        )
        .as_str(),
    );

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/html".try_into()?);

    Ok((headers, subscription_html))
}

#[debug_handler]
pub async fn get_index_html() -> Result<impl IntoResponse, super::error::AppError> {
    let index_html = include_str!("../html/index.html");

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/html".try_into()?);

    Ok((StatusCode::OK, headers, index_html))
}

#[debug_handler]
pub async fn get_style_css() -> Result<impl IntoResponse, super::error::AppError> {
    let style_css = include_str!("../html/style.css");

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/css".try_into()?);

    Ok((StatusCode::OK, headers, style_css))
}

#[debug_handler]
pub async fn redirect_to_nojs() -> Result<impl IntoResponse, super::error::AppError> {
    let mut headers = HeaderMap::new();
    headers.insert(header::LOCATION, "./nojs/index".try_into()?);

    Ok((StatusCode::MOVED_PERMANENTLY, headers, ""))
}
