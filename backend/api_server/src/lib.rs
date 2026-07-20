use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    routing::get,
    Json, Router,
};
use diachronic_analysis::AnalysisService;
use example_ranking::ExampleRankingService;
use kernel::EventBus;
use publication::{PublicationError, PublicationService};
use review::ReviewService;
use search::OpenSearchClient;
use serde::Serialize;
use sense_discovery::JobQueue;
use sqlx::PgPool;
use sync::SyncService;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub event_bus: EventBus,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    database: &'static str,
}

pub fn app(
    pool: PgPool,
    opensearch: OpenSearchClient,
    sync_signing_private_key: &str,
) -> Result<Router, PublicationError> {
    let state = AppState {
        pool: pool.clone(),
        event_bus: EventBus::new(16),
    };

    let review_service = Arc::new(ReviewService::default());
    let publication_service = Arc::new(PublicationService::new(
        review_service.clone(),
        sync_signing_private_key,
    )?);
    let sync_service = Arc::new(SyncService::new(publication_service.clone()));

    let api = Router::new()
        .route("/health", get(health).with_state(pool.clone()))
        .merge(identity::router(pool.clone()))
        .merge(corpus_ingestion::routes(pool.clone()))
        .merge(source_verification::routes(pool.clone()))
        .merge(dictionary_import::routes(pool.clone()))
        .merge(model_registry::routes(pool.clone()))
        .merge(audit::routes::routes(pool.clone()))
        .merge(text_processing::routes())
        .merge(card_management::routes(
            pool.clone(),
            state.event_bus.clone(),
        ))
        .merge(search::routes::routes_with_client(pool.clone(), opensearch))
        .merge(sense_matching::routes(
            pool.clone(),
            state.event_bus.clone(),
        ))
        .merge(sense_discovery::routes(
            pool.clone(),
            JobQueue::new(pool.clone()),
        ))
        .merge(diachronic_analysis::routes(Arc::new(
            AnalysisService::default(),
        )))
        .merge(example_ranking::routes(Arc::new(
            ExampleRankingService::default(),
        )))
        .merge(review::routes(review_service))
        .merge(publication::routes(publication_service))
        .merge(sync::routes(sync_service));

    Ok(Router::new().nest("/api/v1", api))
}

async fn health(
    State(pool): State<PgPool>,
) -> Result<Json<HealthResponse>, (StatusCode, &'static str)> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&pool)
        .await
        .map_err(|_| (StatusCode::SERVICE_UNAVAILABLE, "database unavailable"))?;
    Ok(Json(HealthResponse {
        status: "ok",
        database: "ok",
    }))
}

#[cfg(test)]
mod tests;
