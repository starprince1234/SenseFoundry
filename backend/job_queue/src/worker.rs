use kernel::AppError;
use sqlx::PgPool;
use tokio::time::{sleep, Duration};

use crate::{ack, dequeue, fail, ProcessingJob};

pub async fn run_worker(pool: PgPool, worker_id: String) {
    loop {
        match dequeue(&pool, &worker_id).await {
            Ok(Some(job)) => {
                tracing::info!(
                    worker_id,
                    job_id = %job.id,
                    job_type = %job.job_type,
                    "processing job"
                );
                match dispatch_job(&pool, &job).await {
                    Ok(()) => {
                        if let Err(error) = ack(&pool, job.id).await {
                            tracing::error!(job_id = %job.id, %error, "failed to acknowledge job");
                        }
                    }
                    Err(error) => {
                        if let Err(queue_error) =
                            fail(&pool, job.id, &error.to_string(), job.max_attempts).await
                        {
                            tracing::error!(
                                job_id = %job.id,
                                %queue_error,
                                "failed to record job failure"
                            );
                        }
                    }
                }
            }
            Ok(None) => sleep(Duration::from_millis(500)).await,
            Err(error) => {
                tracing::error!(worker_id, %error, "queue dequeue error");
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn dispatch_job(_pool: &PgPool, job: &ProcessingJob) -> Result<(), AppError> {
    match job.job_type.as_str() {
        "embed_card" | "cluster_headword" => Ok(()),
        unknown => Err(AppError::Unprocessable(format!(
            "Unknown job type: {unknown}"
        ))),
    }
}
