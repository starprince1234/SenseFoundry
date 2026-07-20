use thiserror::Error;

use crate::CreateSubmissionRequest;

pub const MAX_TEXT_CHARS: usize = 50_000;
pub const MAX_FILE_BYTES: usize = 10 * 1024 * 1024;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("submission_type must be either 'text' or 'url'")]
    InvalidSubmissionType,
    #[error("text submissions require text and must not include source_url")]
    InvalidTextPayload,
    #[error("url submissions require source_url and must not include text")]
    InvalidUrlPayload,
    #[error("text exceeds the maximum length of {MAX_TEXT_CHARS} characters")]
    TextTooLong,
    #[error("file exceeds the maximum size of {MAX_FILE_BYTES} bytes")]
    FileTooLarge,
    #[error("malicious content detected: {0}")]
    MaliciousContent(&'static str),
}

pub fn validate_submission(request: &CreateSubmissionRequest) -> Result<(), ValidationError> {
    match request.submission_type.as_str() {
        "text" if request.text.is_some() && request.source_url.is_none() => {
            validate_text(request.text.as_deref().unwrap_or_default())
        }
        "text" => Err(ValidationError::InvalidTextPayload),
        "url" if request.source_url.is_some() && request.text.is_none() => Ok(()),
        "url" => Err(ValidationError::InvalidUrlPayload),
        _ => Err(ValidationError::InvalidSubmissionType),
    }
}

pub fn validate_text(text: &str) -> Result<(), ValidationError> {
    if text.chars().count() > MAX_TEXT_CHARS {
        return Err(ValidationError::TextTooLong);
    }

    let normalized = text.to_ascii_uppercase();
    if normalized.contains("<SCRIPT") {
        return Err(ValidationError::MaliciousContent("script tag"));
    }
    if normalized.contains("DROP TABLE") {
        return Err(ValidationError::MaliciousContent("SQL DROP TABLE pattern"));
    }
    if normalized.contains("'; --") {
        return Err(ValidationError::MaliciousContent(
            "SQL comment injection pattern",
        ));
    }

    Ok(())
}

pub fn validate_file_size(size: usize) -> Result<(), ValidationError> {
    if size > MAX_FILE_BYTES {
        Err(ValidationError::FileTooLarge)
    } else {
        Ok(())
    }
}
