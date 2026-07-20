use std::sync::Arc;

use tokio::sync::broadcast;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    SubmissionReceived { submission_id: uuid::Uuid },
    CardVerified { card_id: uuid::Uuid },
    ClusterCompleted { run_id: uuid::Uuid },
    DefinitionDrafted { draft_id: uuid::Uuid },
    PublicationReady { edition_id: uuid::Uuid },
}

#[derive(Clone)]
pub struct EventBus {
    sender: Arc<broadcast::Sender<DomainEvent>>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender: Arc::new(sender),
        }
    }

    pub fn publish(&self, event: DomainEvent) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DomainEvent> {
        self.sender.subscribe()
    }
}
