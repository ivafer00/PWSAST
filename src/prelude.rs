use axum::Json;
use axum::response::{IntoResponse, Response};
use hyper::StatusCode;
use serde_json::json;
pub use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum ApiError {
    InternalServerError,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_msg) = match self {
            Self::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
            ),
        };

        (status, Json(json!({"error": error_msg}))).into_response()



    }
}