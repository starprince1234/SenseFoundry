use axum::{routing::post, Json, Router};
use kernel::{AppError, AppResult};

use crate::{process_text, ProcessTextRequest, ProcessTextResponse};

pub fn routes() -> Router {
    Router::new().route("/text/process", post(process))
}

async fn process(Json(request): Json<ProcessTextRequest>) -> AppResult<Json<ProcessTextResponse>> {
    if request.text.trim().is_empty() {
        return Err(AppError::Unprocessable("text must not be empty".into()));
    }
    if request
        .target_headword
        .as_deref()
        .is_some_and(|headword| headword.trim().is_empty())
    {
        return Err(AppError::Unprocessable(
            "target_headword must not be empty when provided".into(),
        ));
    }

    Ok(Json(process_text(request)))
}
