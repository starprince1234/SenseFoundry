use std::sync::Arc;

use anyhow::Context;
use axum::{extract::{Query, State}, routing::get, Json, Router};
use kernel::{AppError, AppResult};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    hnsw_search, reciprocal_rank_fusion, FusedResult, OpenSearchClient, SENTENCES_INDEX,
};

const DEFAULT_TOP_K: i64 = 20;
const MAX_TOP_K: i64 = 100;
const RRF_K: f64 = 60.0;

#[derive(Clone)]
struct SearchState {
    pool: PgPool,
    opensearch: Arc<OpenSearchClient>,
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    target_headword: String,
    query_text: Option<String>,
    /// JSON array, for example `[0.1,0.2,0.3]` (URL encoded in a GET request).
    query_vector: Option<String>,
    top_k: Option<i64>,
}

pub fn routes(pool: PgPool) -> AppResult<Router> {
    let opensearch_url = std::env::var("OPENSEARCH_URL")
        .context("OPENSEARCH_URL must be set")?;
    Ok(routes_with_client(
        pool,
        OpenSearchClient::new(&opensearch_url),
    ))
}

pub fn routes_with_client(pool: PgPool, opensearch: OpenSearchClient) -> Router {
    let state = SearchState {
        pool,
        opensearch: Arc::new(opensearch),
    };
    Router::new().route("/search", get(search)).with_state(state)
}

async fn search(
    State(state): State<SearchState>,
    Query(query): Query<SearchQuery>,
) -> AppResult<Json<Vec<FusedResult>>> {
    let target_headword = query.target_headword.trim();
    if target_headword.is_empty() {
        return Err(AppError::Unprocessable(
            "target_headword must not be empty".into(),
        ));
    }
    let top_k = query.top_k.unwrap_or(DEFAULT_TOP_K);
    if !(1..=MAX_TOP_K).contains(&top_k) {
        return Err(AppError::Unprocessable(format!(
            "top_k must be between 1 and {MAX_TOP_K}"
        )));
    }

    let bm25_hits = state
        .opensearch
        .search_sentences(
            SENTENCES_INDEX,
            target_headword,
            query.query_text.as_deref(),
            top_k as usize,
        )
        .await?;
    let vector_hits = match query.query_vector {
        Some(raw_vector) => {
            let vector: Vec<f32> = serde_json::from_str(&raw_vector)
                .map_err(|_| AppError::Unprocessable("query_vector must be a JSON array".into()))?;
            if vector.is_empty() {
                return Err(AppError::Unprocessable(
                    "query_vector must not be empty".into(),
                ));
            }
            hnsw_search(&state.pool, &vector, target_headword, top_k).await?
        }
        None => Vec::new(),
    };

    let mut results = reciprocal_rank_fusion(&bm25_hits, &vector_hits, RRF_K);
    results.truncate(top_k as usize);
    Ok(Json(results))
}
