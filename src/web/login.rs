use std::{collections::HashMap, any};
use std::error::Error;
use axum::{Json, extract::State, body::Bytes, extract::Form};
use axum_macros::debug_handler;
use uuid::Uuid;
use serde::{Serialize,Deserialize};
use crate::nju::login::{LoginCredential,LoginOperation};
use super::server::AppState;
use std::sync::Arc;
use super::error::AppError;
use base64::{engine::general_purpose::STANDARD as base64, Engine};
use std::ops::Deref;

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
    let LoginOperation::WaitingVerificationCode { client, captcha, context }=&auth.sessions.get(&sess).ok_or("The session we just created doesn't exist??").map_err(anyhow::Error::msg)? else{
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
) -> Result<Json<LoginStage2Response>,AppError>{
    let mut auth=&mut state.auth.lock().await;
    let op=auth.sessions.get(&session).ok_or("Invalid session").map_err(anyhow::Error::msg)?;
    let cred=op.finish(&username, &password, &captcha_answer).await?;
    let LoginOperation::Done(cred)=cred else{
        Err("The LoginOperation after finish() isn't done").map_err(anyhow::Error::msg)?
    };


    let cookie_db=&state.clone().cookie_db;
    let mut cookie_db=cookie_db.lock().await;

    cookie_db.insert(session.clone(), cred.castgc).await?;

    auth.sessions.remove(&session);

    Ok(Json(LoginStage2Response{
        uuid: session.clone()
    }))
}


pub struct Authenticator {
    sessions: HashMap<String, LoginOperation>,
}

impl Authenticator {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
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

