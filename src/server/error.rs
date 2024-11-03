// Below code is taken from axum example, licensed under MIT
// Enable `?` error handling on handlers

use axum::{
    http::{Response, StatusCode},
    response::IntoResponse,
};
use log::error;

use anyhow::Error;

// Make our own error that wraps `anyhow::Error`.
#[derive(Debug)]
pub struct AppError(Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response<axum::body::Body> {
        error!("Error: {:?}\n{}", self.0, self.0.backtrace());

        let err = include_str!("../html/error.html");
        let err = err.replace("ERROR", self.0.to_string().as_str());

        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(err.to_string().into())
            .unwrap()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
