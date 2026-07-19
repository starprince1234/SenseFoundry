use axum::{extract::State, routing::post, Json, Router};
use kernel::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{candidate, orchestrator, JobQueue, SenseCandidate};

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    job_queue: JobQueue,
}

#[derive(Debug, Deserialize)]
struct ClusterHeadwordRequest {
    headword: String,
}

#[derive(Debug, Serialize)]
struct ClusterHeadwordResponse {
    job_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct ProposeCandidateRequest {
    cluster_id: Uuid,
}

pub fn routes(pool: PgPool, job_queue: JobQueue) -> Router {
    Router::new()
        .route("/cluster-headword", post(cluster_headword))
        .route("/propose-candidate", post(propose_candidate))
        .with_state(AppState { pool, job_queue })
}

async fn cluster_headword(
    State(state): State<AppState>,
    Json(request): Json<ClusterHeadwordRequest>,
) -> AppResult<Json<ClusterHeadwordResponse>> {
    if request.headword.trim().is_empty() {
        return Err(AppError::Unprocessable("headword must not be empty".into()));
    }

    let job_id = orchestrator::trigger_clustering(
        &state.pool,
        request.headword.trim(),
        &state.job_queue,
    )
    .await?;
    Ok(Json(ClusterHeadwordResponse { job_id }))
}

async fn propose_candidate(
    State(state): State<AppState>,
    Json(request): Json<ProposeCandidateRequest>,
) -> AppResult<Json<SenseCandidate>> {
    candidate::propose_sense_candidate(&state.pool, request.cluster_id)
        .await
        .map(Json)
}
