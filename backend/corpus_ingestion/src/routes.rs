use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use kernel::{idempotency::IDEMPOTENCY_KEY_HEADER, AppError, AppResult, Page, PageParams};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    CreateSubmissionOutcome, CreateSubmissionRequest, Submission, SubmissionService,
};

const SUBMITTER_ID_HEADER: &str = "x-submitter-id";

pub fn routes(pool: PgPool) -> Router {
    Router::new()
        .route(
            "/submissions",
            post(create_submission).get(list_submissions),
        )
        .route("/submissions/:id", get(get_submission))
        .with_state(pool)
}

async fn create_submission(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Json(request): Json<CreateSubmissionRequest>,
) -> AppResult<Response> {
    let idempotency_key = required_header(&headers, IDEMPOTENCY_KEY_HEADER)?;
    let submitter_id = required_header(&headers, SUBMITTER_ID_HEADER)?
        .parse::<Uuid>()
        .map_err(|_| AppError::Unprocessable("x-submitter-id must be a UUID".into()))?;
    let outcome = SubmissionService::new(pool)
        .create(submitter_id, idempotency_key, &request)
        .await?;

    Ok(match outcome {
        CreateSubmissionOutcome::Created(submission) => {
            (StatusCode::CREATED, Json(submission)).into_response()
        }
        CreateSubmissionOutcome::Existing(submission) => {
            (StatusCode::OK, Json(submission)).into_response()
        }
    })
}

async fn list_submissions(
    State(pool): State<PgPool>,
    Query(params): Query<PageParams>,
) -> AppResult<Json<Page<Submission>>> {
    SubmissionService::new(pool)
        .list(&params)
        .await
        .map(Json)
}

async fn get_submission(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Submission>> {
    SubmissionService::new(pool).get(id).await.map(Json)
}

fn required_header<'a>(headers: &'a HeaderMap, name: &str) -> AppResult<&'a str> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError::Unprocessable(format!("missing {name} header")))
}
