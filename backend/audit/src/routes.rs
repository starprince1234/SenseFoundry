use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use kernel::{
    error::{AppError, AppResult},
    pagination::PageParams,
};
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

pub fn routes(pool: PgPool) -> Router {
    Router::new()
        .route("/audit-logs", get(list_audit_logs))
        .route("/audit-logs/:id", get(get_audit_log))
        .with_state(pool)
}

async fn list_audit_logs(
    State(pool): State<PgPool>,
    Query(params): Query<PageParams>,
) -> AppResult<Json<serde_json::Value>> {
    let rows = sqlx::query(
        "SELECT id, table_name, row_id, operation, actor_id, new_data, occurred_at
         FROM audit_logs ORDER BY occurred_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(params.limit())
    .bind(params.offset())
    .fetch_all(&pool)
    .await
    .map_err(AppError::Database)?;

    let items: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.try_get::<Uuid, _>("id").ok(),
                "table_name": r.try_get::<String, _>("table_name").ok(),
                "row_id": r.try_get::<Uuid, _>("row_id").ok(),
                "operation": r.try_get::<String, _>("operation").ok(),
                "actor_id": r.try_get::<Option<Uuid>, _>("actor_id").ok().flatten(),
                "new_data": r.try_get::<serde_json::Value, _>("new_data").ok(),
                "occurred_at": r.try_get::<chrono::DateTime<chrono::Utc>, _>("occurred_at").ok(),
            })
        })
        .collect();
    Ok(Json(serde_json::json!({"items": items})))
}

async fn get_audit_log(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let row = sqlx::query(
        "SELECT id, table_name, row_id, operation, actor_id, new_data, occurred_at
         FROM audit_logs WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound(format!("audit log {id}")))?;

    Ok(Json(serde_json::json!({
        "id": row.try_get::<Uuid, _>("id").ok(),
        "table_name": row.try_get::<String, _>("table_name").ok(),
        "row_id": row.try_get::<Uuid, _>("row_id").ok(),
        "operation": row.try_get::<String, _>("operation").ok(),
        "actor_id": row.try_get::<Option<Uuid>, _>("actor_id").ok().flatten(),
        "new_data": row.try_get::<serde_json::Value, _>("new_data").ok(),
        "occurred_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("occurred_at").ok(),
    })))
}
