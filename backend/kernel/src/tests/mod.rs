use axum::{body::to_bytes, response::IntoResponse};
use serde_json::json;
use uuid::Uuid;

use crate::{
    error::ErrorResponse,
    etag::{check_if_match, compute_etag},
    events::DomainEvent,
    idempotency::{IdempotencyRecord, IdempotencyStore},
    AppError, EventBus, PageParams,
};

#[tokio::test]
async fn not_found_error_maps_to_404_response() {
    let response = AppError::NotFound("foo".into()).into_response();

    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("error response body should be readable");
    let error: ErrorResponse =
        serde_json::from_slice(&body).expect("error response should contain valid JSON");
    assert_eq!(error.code, "NOT_FOUND");
    assert_eq!(error.message, "foo");
    assert!(Uuid::parse_str(&error.trace_id).is_ok());
}

#[test]
fn page_params_calculate_offset_and_limit() {
    let params = PageParams {
        page: 2,
        page_size: 10,
        cursor: None,
    };

    assert_eq!(params.offset(), 10);
    assert_eq!(params.limit(), 10);
}

#[test]
fn page_params_handle_zero_page_without_underflow() {
    let params = PageParams {
        page: 0,
        page_size: 10,
        cursor: None,
    };

    assert_eq!(params.offset(), 0);
}

#[tokio::test]
async fn event_bus_delivers_published_event_to_subscriber() {
    let bus = EventBus::new(16);
    let mut receiver = bus.subscribe();
    let submission_id = Uuid::new_v4();

    bus.publish(DomainEvent::SubmissionReceived { submission_id });

    let received = receiver
        .recv()
        .await
        .expect("subscriber should receive the published event");
    assert_eq!(
        received,
        DomainEvent::SubmissionReceived { submission_id }
    );
}

#[test]
fn compute_etag_returns_consistent_sha256_hex() {
    let expected =
        "\"2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824\"";

    assert_eq!(compute_etag(b"hello"), expected);
    assert_eq!(compute_etag(b"hello"), compute_etag(b"hello"));
}

#[test]
fn if_match_accepts_absent_matching_and_wildcard_headers() {
    let etag = compute_etag(b"hello");

    assert!(check_if_match(&etag, None));
    assert!(check_if_match(&etag, Some(&etag)));
    assert!(check_if_match(&etag, Some("*")));
    assert!(!check_if_match(&etag, Some("\"different\"")));
}

#[test]
fn idempotency_store_returns_inserted_record() {
    let store = IdempotencyStore::new();
    let record = IdempotencyRecord {
        key: "request-123".into(),
        response_body: json!({"created": true}),
        status_code: 201,
    };

    store.insert(record);

    let stored = store
        .get("request-123")
        .expect("inserted idempotency record should be available");
    assert_eq!(stored.key, "request-123");
    assert_eq!(stored.response_body, json!({"created": true}));
    assert_eq!(stored.status_code, 201);
}
