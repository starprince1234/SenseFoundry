use chrono::{DateTime, Utc};
use kernel::AppError;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{JobStatus, ProcessingJob};

#[derive(Debug, Clone)]
pub struct EnqueueJob {
    pub job_type: String,
    pub payload: Value,
    pub max_attempts: i32,
    pub idempotency_key: String,
    pub random_seed: Option<i32>,
    pub scheduled_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct ProcessingJobRow {
    id: Uuid,
    job_type: String,
    payload: Value,
    status: JobStatus,
    #[sqlx(rename = "attempts")]
    attempt_count: i32,
    max_attempts: i32,
    idempotency_key: String,
    random_seed: Option<i32>,
    #[sqlx(rename = "error_message")]
    last_error: Option<String>,
    #[sqlx(rename = "run_at")]
    scheduled_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl From<ProcessingJobRow> for ProcessingJob {
    fn from(row: ProcessingJobRow) -> Self {
        Self {
            id: row.id,
            job_type: row.job_type,
            payload: row.payload,
            status: row.status,
            attempt_count: row.attempt_count,
            max_attempts: row.max_attempts,
            idempotency_key: row.idempotency_key,
            random_seed: row.random_seed,
            last_error: row.last_error,
            scheduled_at: row.scheduled_at,
            started_at: row.started_at,
            completed_at: row.completed_at,
            created_at: row.created_at,
        }
    }
}

const RETURNING_COLUMNS: &str = r#"
    id, job_type, payload, status, attempts, max_attempts, idempotency_key,
    random_seed, error_message, COALESCE(run_at, created_at) AS run_at,
    started_at, completed_at, created_at
"#;

pub async fn enqueue(pool: &PgPool, input: EnqueueJob) -> Result<ProcessingJob, AppError> {
    if input.job_type.trim().is_empty() {
        return Err(AppError::Unprocessable("job type must not be empty".into()));
    }
    if input.idempotency_key.trim().is_empty() {
        return Err(AppError::Unprocessable(
            "idempotency key must not be empty".into(),
        ));
    }
    if input.max_attempts <= 0 {
        return Err(AppError::Unprocessable(
            "max attempts must be greater than zero".into(),
        ));
    }

    let query = format!(
        r#"
        INSERT INTO processing_jobs
            (job_type, payload, status, attempts, max_attempts, random_seed, run_at,
             idempotency_key)
        VALUES ($1, $2, 'QUEUED', 0, $3, $4, $5, $6)
        ON CONFLICT (idempotency_key) DO UPDATE
        SET idempotency_key = EXCLUDED.idempotency_key
        RETURNING {RETURNING_COLUMNS}
        "#
    );
    let row = sqlx::query_as::<_, ProcessingJobRow>(&query)
        .bind(input.job_type)
        .bind(input.payload)
        .bind(input.max_attempts)
        .bind(input.random_seed)
        .bind(input.scheduled_at)
        .bind(input.idempotency_key)
        .fetch_one(pool)
        .await?;

    Ok(row.into())
}

pub async fn dequeue(
    pool: &PgPool,
    worker_id: &str,
) -> Result<Option<ProcessingJob>, AppError> {
    let query = format!(
        r#"
        UPDATE processing_jobs
        SET status = 'RUNNING',
            started_at = NOW(),
            attempts = attempts + 1,
            updated_at = NOW()
        WHERE id = (
            SELECT id FROM processing_jobs
            WHERE status IN ('QUEUED', 'RETRYING')
              AND COALESCE(run_at, created_at) <= NOW()
              AND deleted_at IS NULL
            ORDER BY COALESCE(run_at, created_at) ASC, created_at ASC
            FOR UPDATE SKIP LOCKED
            LIMIT 1
        )
        RETURNING {RETURNING_COLUMNS}
        "#
    );
    let job = sqlx::query_as::<_, ProcessingJobRow>(&query)
        .fetch_optional(pool)
        .await?;

    if let Some(ref row) = job {
        tracing::debug!(worker_id, job_id = %row.id, "worker claimed job");
    }
    Ok(job.map(Into::into))
}

pub async fn ack(pool: &PgPool, job_id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE processing_jobs \
         SET status = 'SUCCEEDED', completed_at = NOW(), updated_at = NOW() \
         WHERE id = $1 AND status = 'RUNNING' AND deleted_at IS NULL",
    )
    .bind(job_id)
    .execute(pool)
    .await?;

    ensure_updated(result.rows_affected(), job_id, "acknowledge")
}

pub async fn fail(
    pool: &PgPool,
    job_id: Uuid,
    error: &str,
    max_attempts: i32,
) -> Result<(), AppError> {
    let result = sqlx::query(
        r#"
        UPDATE processing_jobs
        SET status = CASE
                WHEN attempts < LEAST(max_attempts, $2) THEN 'RETRYING'
                ELSE 'DEAD_LETTER'
            END,
            error_message = $3,
            run_at = CASE
                WHEN attempts < LEAST(max_attempts, $2)
                    THEN NOW() + (attempts * INTERVAL '30 seconds')
                ELSE run_at
            END,
            completed_at = CASE
                WHEN attempts >= LEAST(max_attempts, $2) THEN NOW()
                ELSE NULL
            END,
            updated_at = NOW()
        WHERE id = $1 AND status = 'RUNNING' AND deleted_at IS NULL
        "#,
    )
    .bind(job_id)
    .bind(max_attempts)
    .bind(error)
    .execute(pool)
    .await?;

    ensure_updated(result.rows_affected(), job_id, "fail")
}

fn ensure_updated(rows_affected: u64, job_id: Uuid, operation: &str) -> Result<(), AppError> {
    if rows_affected == 1 {
        Ok(())
    } else {
        Err(AppError::Conflict(format!(
            "cannot {operation} job {job_id} unless it is running"
        )))
    }
}
