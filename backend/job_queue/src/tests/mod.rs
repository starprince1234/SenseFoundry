use chrono::Utc;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{dequeue, enqueue, EnqueueJob, JobStatus, ProcessingJob};

#[test]
fn test_queued_can_transition_to_running() {
    assert!(JobStatus::Queued.can_transition_to(&JobStatus::Running));
}

#[test]
fn test_running_cannot_transition_to_queued() {
    assert!(!JobStatus::Running.can_transition_to(&JobStatus::Queued));
}

#[test]
fn test_failed_goes_to_dead_letter_at_max_attempts() {
    assert!(JobStatus::Failed.can_transition_to(&JobStatus::DeadLetter));
}

#[test]
fn test_random_seed_field_present() {
    let job = ProcessingJob {
        id: Uuid::new_v4(),
        job_type: "cluster_headword".into(),
        payload: serde_json::json!({}),
        status: JobStatus::Queued,
        attempt_count: 0,
        max_attempts: 3,
        idempotency_key: "test-key".into(),
        random_seed: Some(42),
        last_error: None,
        scheduled_at: Utc::now(),
        started_at: None,
        completed_at: None,
        created_at: Utc::now(),
    };

    assert_eq!(job.random_seed, Some(42));
}

#[sqlx::test]
#[ignore = "requires a PostgreSQL test database"]
async fn concurrent_workers_claim_distinct_jobs(pool: PgPool) -> sqlx::Result<()> {
    create_processing_jobs_table(&pool).await?;
    for sequence in 0..2 {
        enqueue(
            &pool,
            EnqueueJob {
                job_type: "embed_card".into(),
                payload: serde_json::json!({"sequence": sequence}),
                max_attempts: 3,
                idempotency_key: format!("concurrent-{sequence}"),
                random_seed: None,
                scheduled_at: Utc::now(),
            },
        )
        .await
        .map_err(app_error_to_sqlx)?;
    }

    let (first, second) = tokio::join!(
        dequeue(&pool, "worker-one"),
        dequeue(&pool, "worker-two")
    );
    let first = first.map_err(app_error_to_sqlx)?.expect("first job");
    let second = second.map_err(app_error_to_sqlx)?.expect("second job");

    assert_ne!(first.id, second.id);
    assert_eq!(first.status, JobStatus::Running);
    assert_eq!(second.status, JobStatus::Running);
    assert_eq!(first.attempt_count, 1);
    assert_eq!(second.attempt_count, 1);
    Ok(())
}

#[sqlx::test]
#[ignore = "requires a PostgreSQL test database"]
async fn enqueue_is_idempotent(pool: PgPool) -> sqlx::Result<()> {
    create_processing_jobs_table(&pool).await?;
    let key = "stable-idempotency-key";
    let original = enqueue(
        &pool,
        EnqueueJob {
            job_type: "cluster_headword".into(),
            payload: serde_json::json!({"version": 1}),
            max_attempts: 3,
            idempotency_key: key.into(),
            random_seed: Some(42),
            scheduled_at: Utc::now(),
        },
    )
    .await
    .map_err(app_error_to_sqlx)?;
    let duplicate = enqueue(
        &pool,
        EnqueueJob {
            job_type: "cluster_headword".into(),
            payload: serde_json::json!({"version": 2}),
            max_attempts: 5,
            idempotency_key: key.into(),
            random_seed: Some(99),
            scheduled_at: Utc::now(),
        },
    )
    .await
    .map_err(app_error_to_sqlx)?;

    assert_eq!(duplicate.id, original.id);
    assert_eq!(duplicate.payload, original.payload);
    assert_eq!(duplicate.random_seed, Some(42));
    let count: i64 = sqlx::query("SELECT COUNT(*) AS count FROM processing_jobs")
        .fetch_one(&pool)
        .await?
        .try_get("count")?;
    assert_eq!(count, 1);
    Ok(())
}

fn app_error_to_sqlx(error: kernel::AppError) -> sqlx::Error {
    sqlx::Error::Protocol(error.to_string())
}

async fn create_processing_jobs_table(pool: &PgPool) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE processing_jobs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            job_type TEXT NOT NULL,
            payload JSONB NOT NULL DEFAULT '{}'::jsonb,
            status TEXT NOT NULL CHECK (
                status IN ('QUEUED', 'RUNNING', 'SUCCEEDED', 'FAILED', 'RETRYING', 'DEAD_LETTER')
            ),
            attempts INT NOT NULL DEFAULT 0 CHECK (attempts >= 0),
            max_attempts INT NOT NULL DEFAULT 3 CHECK (max_attempts > 0),
            random_seed INT,
            run_at TIMESTAMPTZ,
            started_at TIMESTAMPTZ,
            completed_at TIMESTAMPTZ,
            error_message TEXT,
            idempotency_key TEXT NOT NULL UNIQUE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            deleted_at TIMESTAMPTZ
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}
