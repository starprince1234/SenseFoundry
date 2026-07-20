use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    response::IntoResponse,
};
use kernel::{AppError, EventBus};
use serde_json::Value;
use tower::ServiceExt;

use crate::{app, AppState};

#[tokio::test]
async fn test_health_route_is_registered() {
    let response = app()
        .oneshot(Request::get("/api/v1/health").body(Body::empty()).unwrap())
        .await
        .expect("health route should be reachable");

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("health body should be readable");
    assert_eq!(&body[..], b"ok");
}

#[test]
fn test_app_state_fields_are_present() {
    let state = AppState {
        pool: (),
        event_bus: EventBus::new(1),
    };

    let AppState { pool, event_bus } = state;
    let _ = pool;
    let _ = event_bus;
}

#[tokio::test]
async fn test_error_response_format() {
    let response = AppError::NotFound("test resource".into()).into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("error body should be readable");
    let json: Value = serde_json::from_slice(&body).expect("error response should be JSON");

    assert_eq!(json["code"], "NOT_FOUND");
    assert_eq!(json["message"], "test resource");
    assert!(json["trace_id"].as_str().is_some());
}

#[test]
fn test_workspace_compiles() {
    let _ = app();
    assert!(true);
}
