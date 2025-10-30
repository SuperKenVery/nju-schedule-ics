use anyhow::Error;
/// Below code is taken from axum example, licensed under MIT
/// Enable `?` error handling on handlers
use axum::{
    http::{Response, StatusCode},
    response::IntoResponse,
};
use tracing::error;
use uuid::Uuid;

/// Make our own error that wraps `anyhow::Error`.
#[derive(Debug)]
pub struct AppError {
    error: Error,
}

/// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response<axum::body::Body> {
        let error_id = Uuid::new_v4().to_string();
        error!(
            "ID={}\nError: {:?}\n{:#?}",
            error_id.as_str(),
            self.error,
            self.error.backtrace()
        );

        let err = format!("错误ID: {}\n错误信息: {}", error_id, self.error.to_string());

        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(err.to_string().into())
            .unwrap()
    }
}

/// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
/// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<Error>,
{
    fn from(err: E) -> Self {
        Self { error: err.into() }
    }
}
