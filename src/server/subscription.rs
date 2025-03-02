use anyhow::anyhow;
use axum::http::{header, HeaderMap, HeaderValue};
use axum::response::IntoResponse;
use axum::{body::Bytes, extract::Path, extract::State};
use axum_macros::debug_handler;
use uuid::Uuid;

use super::error::AppError;
use super::server::AppState;
use std::sync::Arc;

#[debug_handler]
pub async fn get_ical(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let cookie_db = &mut state.cookie_db.lock().await;
    let uuid = uuid.to_string();
    let cred = cookie_db
        .get_cred(&uuid)
        .await
        .ok_or(anyhow!("Invalid UUID"))?;
    let cal =
        crate::schedule::calendar::Calendar::from_login(cred.clone(), &state.clone().hcal).await;

    match cal {
        Ok(cal) => {
            let cal = cal.to_bytes()?;
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str("text/calendar")?,
            );
            Ok((headers, cal))
        }
        Err(e) => {
            // cookie_db.remove_uuid(&uuid).await?;
            Err(e.into())
        }
    }
}

#[allow(dead_code)]
#[debug_handler]
pub async fn test_ical(// State(state): State<Arc<AppState>>,
) -> Result<Bytes, AppError> {
    let cal = crate::schedule::calendar::Calendar::from_test().await?;
    let cal = cal.to_bytes()?;
    Ok(cal.into())
}
