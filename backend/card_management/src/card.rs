use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "UPPERCASE")]
pub enum CardStatus {
    Draft,
    Processing,
    NeedsVerification,
    Verified,
    Matched,
    Clustered,
    Reviewed,
    Archived,
}

impl CardStatus {
    pub fn can_transition_to(&self, next: &CardStatus) -> bool {
        use CardStatus::*;
        matches!(
            (self, next),
            (Draft, Processing)
                | (Processing, NeedsVerification)
                | (NeedsVerification, Verified)
                | (Verified, Matched)
                | (Matched, Clustered)
                | (Clustered, Reviewed)
                | (Reviewed, Archived)
                | (NeedsVerification, Archived)
                | (Verified, NeedsVerification)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CorpusCard {
    pub id: Uuid,
    pub usage_instance_id: Uuid,
    pub target_headword: String,
    pub sentence_text: String,
    pub context_window: Option<String>,
    pub target_spans: serde_json::Value,
    pub status: CardStatus,
    pub quality_score: f32,
    pub annotation: Option<serde_json::Value>,
    pub model_version: Option<String>,
    pub feature_version: Option<String>,
    pub provenance_chain: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationEntry {
    pub annotated_by: Uuid,
    pub annotated_at: DateTime<Utc>,
    pub correction_type: String,
    pub before: serde_json::Value,
    pub after: serde_json::Value,
}

/// Appends an annotation while retaining every valid prior history entry.
pub fn append_annotation(
    existing: &Option<serde_json::Value>,
    new_entry: AnnotationEntry,
) -> serde_json::Value {
    let mut history: Vec<AnnotationEntry> = existing
        .as_ref()
        .and_then(|value| serde_json::from_value(value.clone()).ok())
        .unwrap_or_default();
    history.push(new_entry);
    serde_json::to_value(history).expect("AnnotationEntry serialization cannot fail")
}
