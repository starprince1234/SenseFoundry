use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    response::IntoResponse,
};
use kernel::AppError;
use p256::ecdsa::SigningKey;
use p256::pkcs8::{EncodePrivateKey, LineEnding};
use rand_core::OsRng;
use search::OpenSearchClient;
use serde_json::Value;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower::ServiceExt;

use crate::app;

fn test_pool() -> PgPool {
    PgPoolOptions::new()
        .connect_lazy("postgresql://test:test@localhost/test")
        .expect("test database URL should parse")
}

fn test_signing_key() -> String {
    SigningKey::random(&mut OsRng)
        .to_pkcs8_pem(LineEnding::LF)
        .expect("test key should encode")
        .to_string()
}

#[tokio::test]
async fn test_health_route_is_registered() {
    let response = app(
        test_pool(),
        OpenSearchClient::new("http://localhost:9200"),
        &test_signing_key(),
    )
        .expect("application should initialize")
        .oneshot(Request::get("/api/v1/health").body(Body::empty()).unwrap())
        .await
        .expect("health route should be reachable");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
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

#[tokio::test]
async fn test_workspace_compiles() {
    let _ = app(
        test_pool(),
        OpenSearchClient::new("http://localhost:9200"),
        &test_signing_key(),
    )
    .expect("application should initialize");
    assert!(true);
}
