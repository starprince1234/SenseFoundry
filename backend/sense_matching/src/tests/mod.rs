use uuid::Uuid;

use crate::detect_unknown;

#[test]
fn test_detect_unknown_when_all_scores_low() {
    let scores = vec![(Uuid::new_v4(), 0.1), (Uuid::new_v4(), 0.3)];
    assert!(detect_unknown(&scores));
}

#[test]
fn test_detect_unknown_false_when_one_score_high() {
    let scores = vec![(Uuid::new_v4(), 0.9), (Uuid::new_v4(), 0.2)];
    assert!(!detect_unknown(&scores));
}

#[test]
fn test_empty_rerank_is_unknown() {
    assert!(detect_unknown(&[]));
}

#[test]
fn test_threshold_score_is_not_unknown() {
    let scores = vec![(Uuid::new_v4(), 0.5)];
    assert!(!detect_unknown(&scores));
}
