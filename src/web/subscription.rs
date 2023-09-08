use std::{collections::HashMap, any};
use std::error::Error;
use axum::response::AppendHeaders;
use axum::{Json, extract::State, body::Bytes, extract::Path};
use axum_macros::debug_handler;
use uuid::Uuid;
use serde::{Serialize,Deserialize};
use crate::nju::login::{LoginCredential,LoginOperation};
use super::server::AppState;
use std::sync::Arc;
use super::error::AppError;
use base64::{engine::general_purpose::STANDARD as base64, Engine};
use std::ops::Deref;

#[debug_handler]
pub async fn get_ical(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Bytes,AppError> {
    let cookie_db=&mut state.cookie_db.lock().await;
    let uuid=uuid.to_string();
    let cred=cookie_db.get(&uuid).ok_or("Invalid uuid").map_err(anyhow::Error::msg)?;
    println!("Buliding calendar...");
    let cal=crate::schedule::calendar::Calendar::from_login(cred.clone()).await?;
    let cal=cal.to_bytes()?;
    println!("Done");

    Ok(cal.into())
}

#[debug_handler]
pub async fn test_ical(
    // State(state): State<Arc<AppState>>,
) -> Result<Bytes,AppError> {
    let cal=crate::schedule::calendar::Calendar::from_test().await?;
    let cal=cal.to_bytes()?;
    Ok(cal.into())
}
