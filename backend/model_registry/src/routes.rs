use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use kernel::{AppResult, Page, PageParams};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    service::{self, CreateModel, CreatePromptTemplate, UpdateModel},
    ModelVersion, PromptTemplate,
};

pub fn routes(pool: PgPool) -> Router {
    Router::new()
        .route("/models", get(list_models).post(create_model))
        .route("/models/:id", get(get_model).patch(update_model))
        .route(
            "/prompt-templates",
            get(list_templates).post(create_template),
        )
        .route("/prompt-templates/:id", get(get_template))
        .with_state(pool)
}

async fn create_model(
    State(pool): State<PgPool>,
    Json(request): Json<CreateModel>,
) -> AppResult<Json<ModelVersion>> {
    Ok(Json(service::create_model(&pool, request).await?))
}

async fn list_models(
    State(pool): State<PgPool>,
    Query(params): Query<PageParams>,
) -> AppResult<Json<Page<ModelVersion>>> {
    Ok(Json(service::list_models(&pool, &params).await?))
}

async fn get_model(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ModelVersion>> {
    Ok(Json(service::get_model(&pool, id).await?))
}

async fn update_model(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateModel>,
) -> AppResult<Json<ModelVersion>> {
    Ok(Json(service::update_model(&pool, id, request).await?))
}

async fn create_template(
    State(pool): State<PgPool>,
    Json(request): Json<CreatePromptTemplate>,
) -> AppResult<Json<PromptTemplate>> {
    Ok(Json(service::create_template(&pool, request).await?))
}

async fn list_templates(
    State(pool): State<PgPool>,
    Query(params): Query<PageParams>,
) -> AppResult<Json<Page<PromptTemplate>>> {
    Ok(Json(service::list_templates(&pool, &params).await?))
}

async fn get_template(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<PromptTemplate>> {
    Ok(Json(service::get_template(&pool, id).await?))
}
