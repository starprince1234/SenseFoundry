use std::sync::Arc;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::service::{Example, ExampleRankingService, RankedExample};

pub fn routes(service: Arc<ExampleRankingService>) -> Router {
    Router::new()
        .route("/rank-examples", post(rank))
        .route("/examples", get(list))
        .with_state(service)
}

async fn rank(
    State(service): State<Arc<ExampleRankingService>>,
    Json(examples): Json<Vec<Example>>,
) -> Json<Vec<RankedExample>> {
    Json(service.rank(examples))
}

async fn list(
    State(service): State<Arc<ExampleRankingService>>,
) -> Json<Vec<RankedExample>> {
    Json(service.list())
}
