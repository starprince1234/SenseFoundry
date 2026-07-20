use crate::{
    validator::{
        validate_file_size, validate_submission, validate_text, ValidationError, MAX_FILE_BYTES,
        MAX_TEXT_CHARS,
    },
    CreateSubmissionRequest,
};

#[test]
fn accepts_text_at_character_limit() {
    let text = "字".repeat(MAX_TEXT_CHARS);

    assert_eq!(validate_text(&text), Ok(()));
}

#[test]
fn rejects_text_over_character_limit() {
    let text = "字".repeat(MAX_TEXT_CHARS + 1);

    assert_eq!(validate_text(&text), Err(ValidationError::TextTooLong));
}

#[test]
fn rejects_malicious_patterns_case_insensitively() {
    for (text, reason) in [
        ("<ScRiPt>alert(1)</script>", "script tag"),
        ("drop table corpus_submissions", "SQL DROP TABLE pattern"),
        ("admin'; --", "SQL comment injection pattern"),
    ] {
        assert_eq!(
            validate_text(text),
            Err(ValidationError::MaliciousContent(reason))
        );
    }
}

#[test]
fn enforces_file_size_limit() {
    assert_eq!(validate_file_size(MAX_FILE_BYTES), Ok(()));
    assert_eq!(
        validate_file_size(MAX_FILE_BYTES + 1),
        Err(ValidationError::FileTooLarge)
    );
}

#[test]
fn requires_payload_matching_submission_type() {
    let request = CreateSubmissionRequest {
        text: Some("valid corpus text".into()),
        source_url: Some("https://example.com".into()),
        submission_type: "text".into(),
    };

    assert_eq!(
        validate_submission(&request),
        Err(ValidationError::InvalidTextPayload)
    );
}
