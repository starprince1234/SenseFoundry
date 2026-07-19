use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReviewState { Pending, InProgress, Completed, Expired }

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Decision { Approve, Reject, Split, Merge, Amend, Abstain }

#[derive(Clone, Debug, Serialize)]
pub struct ReviewVote { pub reviewer_id: Uuid, pub decision: Decision, pub decided_at: DateTime<Utc> }

#[derive(Clone, Debug, Serialize)]
pub struct ReviewTask {
    pub id: Uuid,
    pub sense_candidate_id: Uuid,
    pub reviewer_ids: Vec<Uuid>,
    pub arbiter_id: Option<Uuid>,
    pub state: ReviewState,
    pub votes: Vec<ReviewVote>,
    pub publishable: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ReviewError {
    #[error("review task not found")] NotFound,
    #[error("reviewer is not assigned to task")] UnauthorizedReviewer,
    #[error("review task is already closed")] Closed,
}

#[derive(Clone, Default)]
pub struct ReviewService { tasks: Arc<RwLock<HashMap<Uuid, ReviewTask>>> }

impl ReviewService {
    pub fn create(&self, sense_candidate_id: Uuid, reviewer_ids: Vec<Uuid>) -> ReviewTask {
        let task = ReviewTask { id: Uuid::new_v4(), sense_candidate_id, reviewer_ids, arbiter_id: None, state: ReviewState::Pending, votes: vec![], publishable: false };
        self.tasks.write().unwrap_or_else(|e| e.into_inner()).insert(task.id, task.clone());
        task
    }

    pub fn list(&self) -> Vec<ReviewTask> { self.tasks.read().unwrap_or_else(|e| e.into_inner()).values().cloned().collect() }

    pub fn decide(&self, id: Uuid, reviewer_id: Uuid, decision: Decision, arbiter_id: Option<Uuid>) -> Result<ReviewTask, ReviewError> {
        let mut tasks = self.tasks.write().unwrap_or_else(|e| e.into_inner());
        let task = tasks.get_mut(&id).ok_or(ReviewError::NotFound)?;
        if matches!(task.state, ReviewState::Completed | ReviewState::Expired) { return Err(ReviewError::Closed); }
        if !task.reviewer_ids.contains(&reviewer_id) && task.arbiter_id != Some(reviewer_id) { return Err(ReviewError::UnauthorizedReviewer); }
        task.state = ReviewState::InProgress;
        task.votes.retain(|vote| vote.reviewer_id != reviewer_id);
        task.votes.push(ReviewVote { reviewer_id, decision, decided_at: Utc::now() });
        let approvals = task.votes.iter().filter(|vote| vote.decision == Decision::Approve).count();
        let substantive: Vec<_> = task.votes.iter().filter(|vote| vote.decision != Decision::Abstain).map(|vote| vote.decision).collect();
        task.publishable = approvals >= 2;
        if task.publishable { task.state = ReviewState::Completed; }
        else if substantive.len() >= 2 && substantive.windows(2).any(|pair| pair[0] != pair[1]) { task.arbiter_id = arbiter_id; }
        Ok(task.clone())
    }

    pub fn expire(&self, id: Uuid) -> Result<ReviewTask, ReviewError> {
        let mut tasks = self.tasks.write().unwrap_or_else(|e| e.into_inner());
        let task = tasks.get_mut(&id).ok_or(ReviewError::NotFound)?;
        task.state = ReviewState::Expired;
        Ok(task.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn requires_two_approvals_and_assigns_arbiter_on_disagreement() {
        let service = ReviewService::default(); let first = Uuid::new_v4(); let second = Uuid::new_v4(); let arbiter = Uuid::new_v4();
        let task = service.create(Uuid::new_v4(), vec![first, second]);
        service.decide(task.id, first, Decision::Approve, None).expect("first vote");
        let split = service.decide(task.id, second, Decision::Reject, Some(arbiter)).expect("second vote");
        assert_eq!(split.arbiter_id, Some(arbiter)); assert!(!split.publishable);
    }
}
