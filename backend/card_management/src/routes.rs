use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use kernel::{AppResult, EventBus, Page, PageParams};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{service, CardStatus, CorpusCard};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub event_bus: EventBus,
}

#[derive(Debug, Deserialize)]
struct UpdateCardStatus {
    status: CardStatus,
    annotated_by: Uuid,
}

pub fn routes(pool: PgPool, event_bus: EventBus) -> Router {
    Router::new()
        .route("/corpus-cards", get(list_cards))
        .route(
            "/corpus-cards/:id",
            get(get_card).patch(update_card_status),
        )
        .with_state(AppState { pool, event_bus })
}

async fn list_cards(
    State(state): State<AppState>,
    Query(params): Query<PageParams>,
) -> AppResult<Json<Page<CorpusCard>>> {
    service::list_cards(&state.pool, &params).await.map(Json)
}

async fn get_card(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<CorpusCard>> {
    service::get_card(&state.pool, id).await.map(Json)
}

async fn update_card_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateCardStatus>,
) -> AppResult<Json<CorpusCard>> {
    service::update_card_status(&state.pool, id, request.status, request.annotated_by)
        .await
        .map(Json)
}
