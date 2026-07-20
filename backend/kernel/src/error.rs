use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("unprocessable: {0}")]
    Unprocessable(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("internal: {0}")]
    Internal(#[from] anyhow::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    pub trace_id: String,
}

pub type AppResult<T> = Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let trace_id = Uuid::new_v4().to_string();
        let (status, code, message) = match &self {
            AppError::NotFound(message) => {
                (StatusCode::NOT_FOUND, "NOT_FOUND", message.clone())
            }
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Unauthorized".into(),
            ),
            AppError::Forbidden(message) => {
                (StatusCode::FORBIDDEN, "FORBIDDEN", message.clone())
            }
            AppError::Conflict(message) => {
                (StatusCode::CONFLICT, "CONFLICT", message.clone())
            }
            AppError::Unprocessable(message) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "UNPROCESSABLE",
                message.clone(),
            ),
            AppError::Database(error) => {
                tracing::error!(error = %error, trace_id, "database request failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "Database operation failed".into(),
                )
            }
            AppError::Internal(error) => {
                tracing::error!(error = %error, trace_id, "internal request failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Internal server error".into(),
                )
            }
        };

        let body = Json(ErrorResponse {
            code: code.to_string(),
            message,
            details: None,
            trace_id,
        });
        (status, body).into_response()
    }
}
