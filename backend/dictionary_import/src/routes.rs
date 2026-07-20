use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use kernel::{AppResult, Page, PageParams};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    DictionaryImportService, Headword, NewHeadword, NewReferenceSense, ReferenceSense,
};

pub fn routes(pool: PgPool) -> Router {
    Router::new()
        .route("/headwords", post(create_headword).get(list_headwords))
        .route(
            "/reference-senses",
            post(create_reference_sense).get(list_reference_senses),
        )
        .route("/reference-senses/:id", get(get_reference_sense))
        .with_state(pool)
}

async fn create_headword(
    State(pool): State<PgPool>,
    Json(input): Json<NewHeadword>,
) -> AppResult<Json<Headword>> {
    DictionaryImportService::create_headword(&pool, input)
        .await
        .map(Json)
}

async fn list_headwords(
    State(pool): State<PgPool>,
    Query(params): Query<PageParams>,
) -> AppResult<Json<Page<Headword>>> {
    DictionaryImportService::list_headwords(&pool, &params)
        .await
        .map(Json)
}

async fn create_reference_sense(
    State(pool): State<PgPool>,
    Json(input): Json<NewReferenceSense>,
) -> AppResult<Json<ReferenceSense>> {
    DictionaryImportService::create_reference_sense(&pool, input)
        .await
        .map(Json)
}

async fn list_reference_senses(
    State(pool): State<PgPool>,
    Query(params): Query<PageParams>,
) -> AppResult<Json<Page<ReferenceSense>>> {
    DictionaryImportService::list_reference_senses(&pool, &params)
        .await
        .map(Json)
}

async fn get_reference_sense(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ReferenceSense>> {
    DictionaryImportService::get_reference_sense(&pool, id)
        .await
        .map(Json)
}
