pub mod routes;
pub mod service;
pub mod validator;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use routes::routes;
pub use service::{CreateSubmissionOutcome, SubmissionService};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum SubmissionStatus {
    Pending,
    Processing,
    Accepted,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubmissionRequest {
    pub text: Option<String>,
    pub source_url: Option<String>,
    pub submission_type: String,
}

#[derive(Debug, Serialize)]
pub struct Submission {
    pub id: Uuid,
    pub submitter_id: Uuid,
    pub status: SubmissionStatus,
    pub idempotency_key: String,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests;
