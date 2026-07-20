use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditEvent {
    pub id: Uuid,
    pub actor_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub before_state: Option<serde_json::Value>,
    pub after_state: Option<serde_json::Value>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl AuditEvent {
    pub fn new(
        actor_id: Option<Uuid>,
        action: impl Into<String>,
        resource_type: impl Into<String>,
        resource_id: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            actor_id,
            action: action.into(),
            resource_type: resource_type.into(),
            resource_id,
            before_state: None,
            after_state: None,
            metadata: serde_json::Value::Null,
            created_at: Utc::now(),
        }
    }
}

pub fn write_audit_event(event: &AuditEvent) {
    tracing::info!(
        audit_id = %event.id,
        actor_id = ?event.actor_id,
        action = %event.action,
        resource_type = %event.resource_type,
        resource_id = %event.resource_id,
        metadata = %event.metadata,
        "audit event"
    );
}
