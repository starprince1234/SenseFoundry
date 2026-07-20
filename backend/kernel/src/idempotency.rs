use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    http::{header::CONTENT_TYPE, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::Value;

pub const IDEMPOTENCY_KEY_HEADER: &str = "idempotency-key";

#[derive(Clone, Debug)]
pub struct IdempotencyRecord {
    pub key: String,
    pub response_body: Value,
    pub status_code: u16,
}

#[derive(Clone)]
pub struct IdempotencyStore {
    store: Arc<Mutex<HashMap<String, IdempotencyRecord>>>,
}

impl IdempotencyStore {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &str) -> Option<IdempotencyRecord> {
        self.store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(key)
            .cloned()
    }

    pub fn insert(&self, record: IdempotencyRecord) {
        self.store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(record.key.clone(), record);
    }
}

impl Default for IdempotencyStore {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn idempotency_middleware(
    State(store): State<IdempotencyStore>,
    request: Request,
    next: Next,
) -> Response {
    let Some(key) = request
        .headers()
        .get(IDEMPOTENCY_KEY_HEADER)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
    else {
        return next.run(request).await;
    };

    if let Some(record) = store.get(&key) {
        let status = StatusCode::from_u16(record.status_code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        return (status, Json(record.response_body)).into_response();
    }

    let response = next.run(request).await;
    let status = response.status();
    let (mut parts, body) = response.into_parts();
    let body = match to_bytes(body, usize::MAX).await {
        Ok(body) => body,
        Err(error) => {
            tracing::error!(%error, "failed to buffer idempotent response body");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if let Ok(response_body) = serde_json::from_slice(&body) {
        store.insert(IdempotencyRecord {
            key,
            response_body,
            status_code: status.as_u16(),
        });
    }

    if !parts.headers.contains_key(CONTENT_TYPE) {
        parts
            .headers
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    }
    Response::from_parts(parts, Body::from(body))
}
