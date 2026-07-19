use std::sync::Arc; use axum::{extract::{Path, Query, State}, http::StatusCode, routing::get, Json, Router}; use serde::Deserialize; use uuid::Uuid;
use crate::service::{DeltaResponse, SyncManifest, SyncService};
#[derive(Deserialize)] struct DeltaQuery { last_sync_token: u64 }
pub fn routes(service: Arc<SyncService>) -> Router { Router::new().route("/sync-manifests", get(list)).route("/sync-manifests/:id/delta", get(delta)).with_state(service) }
async fn list(State(service): State<Arc<SyncService>>) -> Json<Vec<SyncManifest>> { Json(service.list()) }
async fn delta(State(service): State<Arc<SyncService>>, Path(id): Path<Uuid>, Query(query): Query<DeltaQuery>) -> Result<Json<DeltaResponse>, StatusCode> { service.delta(id, query.last_sync_token).map(Json).ok_or(StatusCode::NOT_FOUND) }
