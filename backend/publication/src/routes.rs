use std::sync::Arc;
use axum::{extract::{Path, State}, http::StatusCode, routing::{get, post}, Json, Router};
use serde::Deserialize; use serde_json::Value; use uuid::Uuid;
use crate::service::{Edition, PublicationError, PublicationService};
#[derive(Deserialize)] struct PublishRequest { headword: String, content: Value, publisher_id: Uuid }
#[derive(Deserialize)] struct RollbackRequest { publisher_id: Uuid }
pub fn routes(service: Arc<PublicationService>) -> Router { Router::new().route("/publications", post(publish)).route("/editions", get(list)).route("/editions/:id/rollback", post(rollback)).with_state(service) }
async fn publish(State(service): State<Arc<PublicationService>>, Json(request): Json<PublishRequest>) -> Result<(StatusCode, Json<Edition>), (StatusCode, String)> { service.publish(&request.headword, request.content, request.publisher_id).map(|edition| (StatusCode::CREATED, Json(edition))).map_err(map_error) }
async fn list(State(service): State<Arc<PublicationService>>) -> Json<Vec<Edition>> { Json(service.list()) }
async fn rollback(State(service): State<Arc<PublicationService>>, Path(id): Path<Uuid>, Json(request): Json<RollbackRequest>) -> Result<(StatusCode, Json<Edition>), (StatusCode, String)> { service.rollback(id, request.publisher_id).map(|edition| (StatusCode::CREATED, Json(edition))).map_err(map_error) }
fn map_error(error: PublicationError) -> (StatusCode, String) { let status = match error { PublicationError::NotFound => StatusCode::NOT_FOUND, PublicationError::Serialization(_) => StatusCode::UNPROCESSABLE_ENTITY }; (status, error.to_string()) }
