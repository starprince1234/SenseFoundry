use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::service::{DefinitionDraft, DraftError, DraftService};

#[derive(Deserialize)]
struct CreateDraftRequest {
    sense_candidate_id: Uuid,
    headword: String,
    pos: String,
}

pub fn routes(service: Arc<DraftService>) -> Router {
    Router::new()
        .route("/definition-drafts", post(create))
        .route("/definition-drafts/:id", get(get_draft))
        .with_state(service)
}

async fn create(
    State(service): State<Arc<DraftService>>,
    Json(request): Json<CreateDraftRequest>,
) -> Result<(StatusCode, Json<DefinitionDraft>), (StatusCode, String)> {
    service
        .create(request.sense_candidate_id, &request.headword, &request.pos)
        .map(|draft| (StatusCode::CREATED, Json(draft)))
        .map_err(map_error)
}

async fn get_draft(
    State(service): State<Arc<DraftService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<DefinitionDraft>, (StatusCode, String)> {
    service.get(id).map(Json).map_err(map_error)
}

fn map_error(error: DraftError) -> (StatusCode, String) {
    let status = match error {
        DraftError::MissingEvidence => StatusCode::UNPROCESSABLE_ENTITY,
        DraftError::NotFound => StatusCode::NOT_FOUND,
        DraftError::Gateway(_) => StatusCode::BAD_GATEWAY,
    };
    (status, error.to_string())
}
