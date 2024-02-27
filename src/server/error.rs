// Below code is taken from axum example, licensed under MIT
// Enable `?` error handling on handlers

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use reqwest::{header, header::HeaderMap};

use anyhow::Error;

// Make our own error that wraps `anyhow::Error`.
#[derive(Debug)]
pub struct AppError(Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        println!("Error: {:?}", self.0);
        let err=std::fs::read_to_string("src/html/error.html").unwrap_or("Failed to load error page. What's more, ERROR".into());
        let err=err.replace("ERROR",self.0.to_string().as_str());

        let mut headers=HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "text/html".try_into().unwrap());

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            headers,
            err,
        )
            .into_response()
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
