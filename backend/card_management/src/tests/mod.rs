use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::{append_annotation, AnnotationEntry, CardStatus};

#[test]
fn test_draft_can_go_to_processing() {
    assert!(CardStatus::Draft.can_transition_to(&CardStatus::Processing));
}

#[test]
fn test_verified_cannot_go_directly_to_clustered() {
    assert!(!CardStatus::Verified.can_transition_to(&CardStatus::Clustered));
}

#[test]
fn test_annotation_history_preserved() {
    let entry = AnnotationEntry {
        annotated_by: Uuid::new_v4(),
        annotated_at: Utc::now(),
        correction_type: "span_correction".into(),
        before: json!({"start": 1}),
        after: json!({"start": 2}),
    };
    let history = append_annotation(&None, entry).unwrap();
    let entries: Vec<AnnotationEntry> = serde_json::from_value(history).unwrap();
    assert_eq!(entries.len(), 1);
}

#[test]
fn test_annotation_appends_not_replaces() {
    let e1 = AnnotationEntry {
        annotated_by: Uuid::new_v4(),
        annotated_at: Utc::now(),
        correction_type: "a".into(),
        before: json!({}),
        after: json!({}),
    };
    let first = append_annotation(&None, e1).unwrap();
    let e2 = AnnotationEntry {
        annotated_by: Uuid::new_v4(),
        annotated_at: Utc::now(),
        correction_type: "b".into(),
        before: json!({}),
        after: json!({}),
    };
    let second = append_annotation(&Some(first), e2).unwrap();
    let entries: Vec<AnnotationEntry> = serde_json::from_value(second).unwrap();
    assert_eq!(entries.len(), 2, "History must accumulate, not be replaced");
}
