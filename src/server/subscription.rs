use super::error::AppError;
use super::state::ServerState;
use crate::adapters::all_school_adapters::school_adapter_from_name;
use crate::adapters::traits::{Login, School};
use anyhow::Context;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum_macros::debug_handler;
use dioxus::prelude::{FromContext, extract};
use std::sync::Arc;

#[debug_handler]
pub async fn get_calendar_file(
    Path((school_adapter, key)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let FromContext(state): FromContext<ServerState> = extract().await?;

    let school: Arc<dyn School> =
        school_adapter_from_name(school_adapter.as_str(), state.db.clone())
            .await
            .context("No such school adapter")?
            .into();
    let cred = school
        .get_cred_from_db(key.as_str())
        .await
        .context("No such key. URL might be wrong.")?;
    let client = school.create_authenticated_client(cred).await?;
    let courses = school.courses(&client).await?;

    todo!()
}
