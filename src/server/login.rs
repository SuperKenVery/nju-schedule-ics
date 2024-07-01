

use axum::response::IntoResponse;
use axum::{Json, extract::State, extract::Form
};
use axum_macros::debug_handler;

use serde::{Serialize,Deserialize};
use super::server::AppState;
use std::sync::Arc;
use super::error::AppError;
use base64::{engine::general_purpose::STANDARD as base64, Engine};



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
    let mut db=state.cookie_db.lock().await;
    let session=db.new_session().await?;
    let captcha=db.get_session_captcha(&session).await?;

    let captcha_base64=base64.encode(captcha);

    Ok(Json(LoginStage1Response{
        session,
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
    let mut db=state.cookie_db.lock().await;
    db.session_login(&session, &username, &password, &captcha_answer).await?;

    Ok(
        Json(LoginStage2Response{
            uuid: session.clone()
        })
    )
}


