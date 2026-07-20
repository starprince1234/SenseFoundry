use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::service::{Edition, PublicationError, PublicationPreview, PublicationService};

#[derive(Deserialize)]
struct PreviewRequest {
    headword: String,
    content: Value,
}

#[derive(Deserialize)]
struct PublishRequest {
    headword: String,
    content: Value,
    publisher_id: Uuid,
    review_task_id: Uuid,
}

#[derive(Deserialize)]
struct RollbackRequest {
    publisher_id: Uuid,
    review_task_id: Uuid,
}

pub fn routes(service: Arc<PublicationService>) -> Router {
    Router::new()
        .route("/publication-preview", post(preview))
        .route("/publications", post(publish))
        .route("/editions", get(list))
        .route("/editions/:id/rollback", post(rollback))
        .with_state(service)
}

async fn preview(
    State(service): State<Arc<PublicationService>>,
    Json(request): Json<PreviewRequest>,
) -> Result<Json<PublicationPreview>, (StatusCode, String)> {
    service
        .preview(&request.headword, &request.content)
        .map(Json)
        .map_err(map_error)
}

async fn publish(
    State(service): State<Arc<PublicationService>>,
    Json(request): Json<PublishRequest>,
) -> Result<(StatusCode, Json<Edition>), (StatusCode, String)> {
    service
        .publish(
            &request.headword,
            request.content,
            request.publisher_id,
            request.review_task_id,
        )
        .map(|edition| (StatusCode::CREATED, Json(edition)))
        .map_err(map_error)
}

async fn list(State(service): State<Arc<PublicationService>>) -> Json<Vec<Edition>> {
    Json(service.list())
}

async fn rollback(
    State(service): State<Arc<PublicationService>>,
    Path(id): Path<Uuid>,
    Json(request): Json<RollbackRequest>,
) -> Result<(StatusCode, Json<Edition>), (StatusCode, String)> {
    service
        .rollback(id, request.publisher_id, request.review_task_id)
        .map(|edition| (StatusCode::CREATED, Json(edition)))
        .map_err(map_error)
}

fn map_error(error: PublicationError) -> (StatusCode, String) {
    let status = match error {
        PublicationError::NotFound | PublicationError::ReviewNotFound => StatusCode::NOT_FOUND,
        PublicationError::ReviewRequired | PublicationError::ReviewedContentMismatch => {
            StatusCode::CONFLICT
        }
        PublicationError::InvalidContent | PublicationError::Serialization(_) => {
            StatusCode::UNPROCESSABLE_ENTITY
        }
        PublicationError::InvalidSigningKey(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, error.to_string())
}
