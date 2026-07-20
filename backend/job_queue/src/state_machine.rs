use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Retrying,
    DeadLetter,
}

impl JobStatus {
    pub fn can_transition_to(&self, next: &JobStatus) -> bool {
        matches!(
            (self, next),
            (JobStatus::Queued, JobStatus::Running)
                | (JobStatus::Running, JobStatus::Succeeded)
                | (JobStatus::Running, JobStatus::Failed)
                | (JobStatus::Failed, JobStatus::Retrying)
                | (JobStatus::Retrying, JobStatus::Running)
                | (JobStatus::Retrying, JobStatus::DeadLetter)
                | (JobStatus::Failed, JobStatus::DeadLetter)
        )
    }
}
