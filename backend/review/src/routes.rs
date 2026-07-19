use std::sync::Arc;
use axum::{extract::{Path, State}, http::StatusCode, routing::{get, post}, Json, Router};
use serde::Deserialize;
use uuid::Uuid;
use crate::service::{Decision, ReviewError, ReviewService, ReviewTask};

#[derive(Deserialize)] struct CreateRequest { sense_candidate_id: Uuid, reviewer_ids: Vec<Uuid> }
#[derive(Deserialize)] struct DecideRequest { reviewer_id: Uuid, decision: Decision, arbiter_id: Option<Uuid> }
pub fn routes(service: Arc<ReviewService>) -> Router { Router::new().route("/review-tasks", post(create).get(list)).route("/review-tasks/:id/decide", post(decide)).with_state(service) }
async fn create(State(service): State<Arc<ReviewService>>, Json(request): Json<CreateRequest>) -> (StatusCode, Json<ReviewTask>) { (StatusCode::CREATED, Json(service.create(request.sense_candidate_id, request.reviewer_ids))) }
async fn list(State(service): State<Arc<ReviewService>>) -> Json<Vec<ReviewTask>> { Json(service.list()) }
async fn decide(State(service): State<Arc<ReviewService>>, Path(id): Path<Uuid>, Json(request): Json<DecideRequest>) -> Result<Json<ReviewTask>, (StatusCode, String)> { service.decide(id, request.reviewer_id, request.decision, request.arbiter_id).map(Json).map_err(map_error) }
fn map_error(error: ReviewError) -> (StatusCode, String) { let status = match error { ReviewError::NotFound => StatusCode::NOT_FOUND, ReviewError::UnauthorizedReviewer => StatusCode::FORBIDDEN, ReviewError::Closed => StatusCode::CONFLICT }; (status, error.to_string()) }
