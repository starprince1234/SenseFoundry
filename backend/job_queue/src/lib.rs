pub mod queue;
pub mod state_machine;
pub mod worker;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use queue::{ack, dequeue, enqueue, fail, EnqueueJob};
pub use state_machine::JobStatus;
pub use worker::run_worker;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingJob {
    pub id: Uuid,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub status: JobStatus,
    pub attempt_count: i32,
    pub max_attempts: i32,
    pub idempotency_key: String,
    pub random_seed: Option<i32>,
    pub last_error: Option<String>,
    pub scheduled_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests;
