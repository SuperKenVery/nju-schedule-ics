use axum::http::{HeaderMap, HeaderValue, header};
use axum::response::IntoResponse;
use axum::{extract::State, body::Bytes, extract::Path};
use axum_macros::debug_handler;
use uuid::Uuid;
use crate::nju::login::LoginCredential;
use super::server::AppState;
use std::sync::Arc;
use super::error::AppError;

#[debug_handler]
pub async fn get_ical(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse,AppError> {
    let cookie_db=&mut state.cookie_db.lock().await;
    let uuid=uuid.to_string();
    let castgc=cookie_db
        .get(&uuid).await?
        .ok_or("Invalid session").map_err(anyhow::Error::msg)?;
    let cred=LoginCredential::new(castgc);
    let cal=crate::schedule::calendar::Calendar::from_login(cred.clone()).await?;
    let cal=cal.to_bytes()?;

    let mut headers=HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_str("text/calendar")?);

    Ok(
        (
            headers,
            cal,
        )
    )
}

#[allow(dead_code)]
#[debug_handler]
pub async fn test_ical(
    // State(state): State<Arc<AppState>>,
) -> Result<Bytes,AppError> {
    let cal=crate::schedule::calendar::Calendar::from_test().await?;
    let cal=cal.to_bytes()?;
    Ok(cal.into())
}
