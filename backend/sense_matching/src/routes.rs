use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use kernel::{AppError, AppResult, EventBus, Page, PageParams};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    add_to_unknown_pool, list_unknown_pool as load_unknown_pool, MatchResult, SenseMatcher,
    UnknownPoolEntry,
};

const UNKNOWN_REASON: &str = "all rerank scores below threshold or no reference senses";

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub event_bus: EventBus,
}

#[derive(Debug, Deserialize)]
struct MatchCardsRequest {
    cards: Vec<MatchCardRequest>,
}

#[derive(Debug, Deserialize)]
struct MatchCardRequest {
    card_id: Uuid,
    target_headword: String,
    h_target: Vec<f32>,
}

#[derive(Debug, Deserialize)]
struct UnknownPoolQuery {
    target_headword: String,
    #[serde(flatten)]
    page: PageParams,
}

pub fn routes(pool: PgPool, event_bus: EventBus) -> Router {
    Router::new()
        .route("/match-cards", post(match_cards))
        .route("/unknown-pool", get(list_unknown_pool))
        .with_state(AppState { pool, event_bus })
}

async fn match_cards(
    State(state): State<AppState>,
    Json(request): Json<MatchCardsRequest>,
) -> AppResult<Json<Vec<MatchResult>>> {
    if request.cards.is_empty() {
        return Err(AppError::Unprocessable(
            "cards must contain at least one item".into(),
        ));
    }

    let inference_url = std::env::var("INFER_SERVICE_URL").map_err(|error| {
        AppError::Internal(anyhow::anyhow!("INFER_SERVICE_URL must be set: {error}"))
    })?;
    let matcher = SenseMatcher::new(state.pool.clone(), inference_url);
    let mut results = Vec::with_capacity(request.cards.len());

    for card in request.cards {
        let target_headword = card.target_headword.trim();
        if target_headword.is_empty() {
            return Err(AppError::Unprocessable(
                "target_headword must not be empty".into(),
            ));
        }
        if card.h_target.is_empty() || card.h_target.iter().any(|value| !value.is_finite()) {
            return Err(AppError::Unprocessable(
                "h_target must contain finite values".into(),
            ));
        }

        let result = matcher
            .match_card(card.card_id, target_headword, &card.h_target)
            .await?;
        persist_match(&state.pool, &result).await?;
        if result.is_unknown {
            // Pool membership is the terminal guard: clustering only consumes matched cards.
            add_to_unknown_pool(&state.pool, result.card_id, UNKNOWN_REASON).await?;
        }
        results.push(result);
    }

    Ok(Json(results))
}

async fn persist_match(pool: &PgPool, result: &MatchResult) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO sense_matches (
            id, corpus_card_id, reference_sense_id, match_score, rerank_score,
            is_unknown, match_method, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'bi_encoder_cross_encoder', NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(result.card_id)
    .bind(result.matched_sense_id)
    .bind(result.match_score.map(f64::from))
    .bind(result.rerank_score.map(f64::from))
    .bind(result.is_unknown)
    .execute(pool)
    .await?;

    if !result.is_unknown {
        sqlx::query(
            "UPDATE corpus_cards SET status = 'MATCHED', updated_at = NOW() WHERE id = $1",
        )
        .bind(result.card_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn list_unknown_pool(
    State(state): State<AppState>,
    Query(query): Query<UnknownPoolQuery>,
) -> AppResult<Json<Page<UnknownPoolEntry>>> {
    let target_headword = query.target_headword.trim();
    if target_headword.is_empty() {
        return Err(AppError::Unprocessable(
            "target_headword must not be empty".into(),
        ));
    }
    load_unknown_pool(&state.pool, target_headword, &query.page)
        .await
        .map(Json)
}
