use axum::{
    extract::{Path, Query, State},
    routing::{get, patch, post},
    Json, Router,
};
use kernel::{AppResult, Page, PageParams};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    service::{self, NewSource, VerificationEvidence},
    Source, SourceKind,
};

#[derive(Debug, Deserialize)]
pub struct CreateSourceRequest {
    pub uri: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub doi: Option<String>,
    pub license: Option<String>,
    pub source_kind: SourceKind,
}

#[derive(Debug, Deserialize)]
pub struct VerifySourceRequest {
    pub isbn: Option<String>,
    pub doi: Option<String>,
    pub license: Option<String>,
    #[serde(default)]
    pub url_accessible: bool,
}

pub fn routes(pool: PgPool) -> Router {
    Router::new()
        .route("/sources", post(create_source))
        .route("/sources", get(list_sources))
        .route("/sources/:id", get(get_source))
        .route("/sources/:id/verify", patch(verify_source))
        .with_state(pool)
}

async fn create_source(
    State(pool): State<PgPool>,
    Json(request): Json<CreateSourceRequest>,
) -> AppResult<Json<Source>> {
    let source = service::create(
        &pool,
        NewSource {
            uri: request.uri,
            title: request.title,
            author: request.author,
            isbn: request.isbn,
            doi: request.doi,
            license: request.license,
            source_kind: request.source_kind,
        },
    )
    .await?;
    Ok(Json(source))
}

async fn list_sources(
    State(pool): State<PgPool>,
    Query(params): Query<PageParams>,
) -> AppResult<Json<Page<Source>>> {
    Ok(Json(service::list(&pool, &params).await?))
}

async fn get_source(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Source>> {
    Ok(Json(service::get(&pool, id).await?))
}

async fn verify_source(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(request): Json<VerifySourceRequest>,
) -> AppResult<Json<Source>> {
    let source = service::verify(
        &pool,
        id,
        VerificationEvidence {
            isbn: request.isbn,
            doi: request.doi,
            license: request.license,
            url_accessible: request.url_accessible,
        },
    )
    .await?;
    Ok(Json(source))
}
