use axum::{extract::State,
    http::{header,HeaderMap, status::StatusCode},
    response::IntoResponse,
    extract::Form
};
use axum_macros::debug_handler;
use serde::Deserialize;
use super::server::AppState;
use super::error::AppError;
use std::sync::Arc;
use crate::nju::login::LoginOperation;

#[debug_handler]
pub async fn get_captcha_content(
    State(state): State<Arc<AppState>>
) -> Result<impl IntoResponse, AppError> {
    let auth=&mut state.auth.lock().await;
    let sess=auth.new_session().await?;
    let LoginOperation::WaitingVerificationCode { client: _, captcha, context: _ }=&auth.sessions.get(&sess).ok_or("The session we just created doesn't exist??").map_err(anyhow::Error::msg)? else{
        Err("The new LoginOperation isn't waiting for verification code").map_err(anyhow::Error::msg)?
    };

    let mut headers=HeaderMap::new();
    headers.insert(header::SET_COOKIE, format!("SESSION={}; SameSite=Strict; HttpOnly;", sess).try_into()?);

    Ok(
        (
            headers,
            captcha.clone()
        )
    )

}

#[derive(Deserialize)]
pub struct LoginForm{
    username: String,
    password: String,
    captcha_answer: String,
}

#[debug_handler]
pub async fn login(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Form(LoginForm{username,password,captcha_answer}): Form<LoginForm>,
) -> Result<impl IntoResponse,AppError>{
    let auth=&mut state.auth.lock().await;
    let cookies=headers.get("cookie").ok_or("No cookie").map_err(anyhow::Error::msg)?.to_str()?;
    let cookies=cookies.split(";")
        .map(|s| {
            let (k,v)=s.split_once("=").ok_or(format!("Invalid cookie: {}", s)).map_err(anyhow::Error::msg)?;
            Ok((k,v))
        })
        .collect::<Result<Vec<(&str,&str)>,anyhow::Error>>()?
        .into_iter();
    let session=cookies
        .filter(|(k,_v)| {
            k==&"SESSION"
        })
        .collect::<Vec<(&str,&str)>>().get(0).ok_or("No SESSION cookie").map_err(anyhow::Error::msg)?.1;
    println!("All sessions: {:#?}, session is {}",auth.sessions.keys(),session);
    let op=auth.sessions.get(session).ok_or("Invalid session").map_err(anyhow::Error::msg)?;
    let cred=op.finish(&username,&password,&captcha_answer).await?;
    let LoginOperation::Done(cred)=cred else{
        Err("The LoginOperation after finish() isn't done").map_err(anyhow::Error::msg)?
    };

    let cookie_db=&state.clone().cookie_db;
    let mut cookie_db=cookie_db.lock().await;

    cookie_db.insert(session.clone(), cred.castgc.clone()).await?;
    auth.sessions.remove(session);

    let subscription_html=std::fs::read_to_string("src/html/subscription.html")?;
    let subscription_html=subscription_html.replace("SUBSCRIPTION_LINK", format!("{}/{}/schedule.ics", state.site_url, session).as_str());

    let mut headers=HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/html".try_into()?);

    Ok(
        (
            headers,
            subscription_html
        )
    )
}

#[debug_handler]
pub async fn get_index_html() -> Result<impl IntoResponse, super::error::AppError> {
    let index_html=std::fs::read_to_string("src/html/index.html")?;

    let mut headers=HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/html".try_into()?);

    Ok(
        (
            StatusCode::OK,
            headers,
            index_html
        )
    )
}
