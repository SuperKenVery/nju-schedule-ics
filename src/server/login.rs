use std::collections::HashMap;
use axum::http::HeaderValue;
use axum::response::IntoResponse;
use axum::{Json, extract::State, extract::Form,
    http::{header,HeaderMap}
};
use axum_macros::debug_handler;
use uuid::Uuid;
use serde::{Serialize,Deserialize};
use super::server::AppState;
use std::sync::Arc;
use super::error::AppError;
use base64::{engine::general_purpose::STANDARD as base64, Engine};
use super::db::CookieDb;
use crate::nju::login::{LoginOperation, LoginCredential};

// Get captcha content
#[derive(Serialize)]
pub struct LoginStage1Response{
    session: String,
    captcha_content: String,
}

#[debug_handler]
pub async fn new_login_session(
    State(state): State<Arc<AppState>>
) -> Result<Json<LoginStage1Response>,AppError> {
    let auth=&mut state.auth.lock().await;
    let sess=auth.new_session().await?;
    let LoginOperation::WaitingVerificationCode { client: _, captcha, context: _ }=&auth.sessions.get(&sess).ok_or("The session we just created doesn't exist??").map_err(anyhow::Error::msg)? else{
        Err("The new LoginOperation isn't waiting for verification code").map_err(anyhow::Error::msg)?
    };

    let captcha_base64=base64.encode(captcha);

    Ok(Json(LoginStage1Response{
        session: sess.clone(),
        captcha_content: captcha_base64,
    }))
}

// Send username and password
#[derive(Deserialize)]
pub struct LoginStage2Request{
    session: String,
    username: String,
    password: String,
    captcha_answer: String,
}

#[derive(Serialize)]
pub struct LoginStage2Response{
    uuid: String,
}

#[debug_handler]
pub async fn finish_login(
    State(state): State<Arc<AppState>>,
    Form(LoginStage2Request{session,username,password,captcha_answer}): Form<LoginStage2Request>,
) -> Result<impl IntoResponse,AppError>{
    let auth=&mut state.auth.lock().await;
    let op=auth.sessions.get(&session).ok_or("Invalid session").map_err(anyhow::Error::msg)?;
    let cred=op.finish(&username, &password, &captcha_answer).await?;
    let LoginOperation::Done(cred)=cred else{
        Err("The LoginOperation after finish() isn't done").map_err(anyhow::Error::msg)?
    };


    let cookie_db=&state.clone().cookie_db;
    let mut cookie_db=cookie_db.lock().await;

    cookie_db.insert(session.clone(), cred.castgc.clone()).await?;

    auth.sessions.remove(&session);

    let mut headers=HeaderMap::new();
    headers.insert(header::SET_COOKIE,HeaderValue::from_str(format!("CASTGC={}",cred.castgc).as_str())?);

    Ok(
        (
            headers,
            Json(LoginStage2Response{
                uuid: session.clone()
            }),
        )
    )
}


pub struct Authenticator {
    pub sessions: HashMap<String, LoginOperation>,
}

impl Authenticator {
    pub async fn new(db: &CookieDb) -> Result<Self,anyhow::Error> {
        let mut sessions=HashMap::new();
        for (k,v) in db.get_all().await? {
            sessions.insert(
                k,
                LoginOperation::Done(LoginCredential::new(v))
            );
        }

        Ok(Self {
            sessions,
        })
    }

    pub async fn new_session(&mut self) -> Result<String,anyhow::Error> {
        let mut sess=Uuid::new_v4().to_string();

        while let Some(_)=self.sessions.get(&sess) {
            // UUID collision
            sess=Uuid::new_v4().to_string();
        }

        let o=LoginOperation::start().await?;
        self.sessions.insert(sess.clone(), o);

        Ok(sess)
    }

}

