use crate::{
    extract_spans, extract_spans_char_fallback, is_injection, is_near_duplicate, normalize,
    normalize_with_variant, split_sentences, ChineseVariant, MatchType, WordToken,
};

#[test]
fn test_da_in_da_dianhua_dual_level() {
    let word_tokens = vec![
        WordToken {
            text: "他".into(),
            start_char: 0,
            end_char: 1,
            pos: None,
        },
        WordToken {
            text: "打电话".into(),
            start_char: 1,
            end_char: 4,
            pos: None,
        },
        WordToken {
            text: "给".into(),
            start_char: 4,
            end_char: 5,
            pos: None,
        },
        WordToken {
            text: "我".into(),
            start_char: 5,
            end_char: 6,
            pos: None,
        },
    ];
    let spans = extract_spans("他打电话给我", "打", &word_tokens);

    assert!(spans
        .iter()
        .any(|span| span.match_type == MatchType::CharInLexeme));
    assert!(spans.iter().any(|span| {
        span.match_type == MatchType::LexemeMultiChar && span.surface == "打电话"
    }));
    assert!(
        spans.len() >= 2,
        "Expected at least 2 spans for 打 in 打电话, got {}",
        spans.len()
    );
}

#[test]
fn test_char_fallback_no_panic_on_garbage() {
    let spans = extract_spans_char_fallback("乱码abc###打!!!", "打", 3);
    assert!(!spans.is_empty());
}

#[test]
fn test_normalize_collapses_whitespace() {
    assert_eq!(normalize("hello   world"), "hello world");
}

#[test]
fn test_normalize_converts_chinese_variant() {
    assert_eq!(
        normalize_with_variant("他打電話給我", ChineseVariant::Simplified),
        "他打电话给我"
    );
}

#[test]
fn test_split_sentences() {
    let sentences = split_sentences("他打电话。我很开心！");
    assert_eq!(sentences, ["他打电话。", "我很开心！"]);
}

#[test]
fn test_injection_detection() {
    assert!(is_injection("<script>alert(1)</script>"));
    assert!(!is_injection("他打电话给我"));
}

#[test]
fn test_near_duplicate_ignores_whitespace_and_punctuation() {
    assert!(is_near_duplicate("他打电话给我。", "他 打电话给我", 0.9));
}
