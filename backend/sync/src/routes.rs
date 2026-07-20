use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{header::CONTENT_TYPE, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::service::{DeltaResponse, SyncManifest, SyncService};

#[derive(Deserialize)]
struct DeltaQuery {
    #[serde(default)]
    last_sync_token: u64,
}

pub fn routes(service: Arc<SyncService>) -> Router {
    Router::new()
        .route("/sync-manifests", get(list))
        .route("/sync-manifests/latest/delta", get(latest_delta))
        .route("/sync-manifests/:id/delta", get(delta))
        .route("/editions/:id/delta", get(edition_delta))
        .with_state(service)
}

async fn list(State(service): State<Arc<SyncService>>) -> Json<Vec<SyncManifest>> {
    Json(service.list())
}

async fn latest_delta(
    State(service): State<Arc<SyncService>>,
    Query(query): Query<DeltaQuery>,
) -> Json<DeltaResponse> {
    Json(service.latest_delta(query.last_sync_token))
}

async fn delta(
    State(service): State<Arc<SyncService>>,
    Path(id): Path<Uuid>,
    Query(query): Query<DeltaQuery>,
) -> Result<Json<DeltaResponse>, StatusCode> {
    service
        .delta(id, query.last_sync_token)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn edition_delta(
    State(service): State<Arc<SyncService>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    service
        .edition_delta(id)
        .map(|bytes| ([(CONTENT_TYPE, "application/json")], bytes))
        .map_err(|_| StatusCode::NOT_FOUND)
}
