use kernel::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct WriteAuditEvent {
    pub table_name: String,
    pub row_id: Uuid,
    pub actor_id: Option<Uuid>,
    pub new_data: serde_json::Value,
}

pub async fn write_audit(pool: &PgPool, event: WriteAuditEvent) -> AppResult<Uuid> {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO audit_logs (id, table_name, row_id, operation, actor_id, new_data, occurred_at)
           VALUES ($1, $2, $3, 'INSERT', $4, $5, NOW())"#,
    )
    .bind(id)
    .bind(event.table_name)
    .bind(event.row_id)
    .bind(event.actor_id)
    .bind(event.new_data)
    .execute(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(id)
}
