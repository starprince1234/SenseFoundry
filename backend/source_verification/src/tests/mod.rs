use crate::{validate_doi, validate_isbn, CopyrightStatus};

#[test]
fn test_copyright_status_verified_requires_license_and_id() {
    let status = CopyrightStatus::determine(true, true, "book", false);
    assert_eq!(status, CopyrightStatus::Verified);
}

#[test]
fn test_url_only_is_rejected() {
    let status = CopyrightStatus::determine(false, false, "web_page", true);
    assert_eq!(status, CopyrightStatus::Rejected);
}

#[test]
fn test_validate_isbn_10() {
    assert!(validate_isbn("0306406152"));
    assert!(!validate_isbn("12345"));
}

#[test]
fn test_validate_doi() {
    assert!(validate_doi("10.1000/182"));
    assert!(!validate_doi("not-a-doi"));
}

#[test]
fn rejected_and_unverifiable_sources_are_not_publishable() {
    assert_eq!(
        CopyrightStatus::Rejected.authorization_flags(),
        (false, false, false)
    );
    assert_eq!(
        CopyrightStatus::Unverifiable.authorization_flags(),
        (false, false, false)
    );
}

#[test]
fn accessible_url_with_license_is_only_partially_verified() {
    let status = CopyrightStatus::determine(true, false, "web_page", true);
    assert_eq!(status, CopyrightStatus::PartiallyVerified);
    assert!(!status.authorization_flags().2);
}
