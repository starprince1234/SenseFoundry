pub mod normalizer;
pub mod quality;
pub mod routes;
pub mod sentence_splitter;
pub mod span_extractor;

use serde::{Deserialize, Serialize};

pub use normalizer::{
    normalize, normalize_with_variant, to_simplified, to_traditional, ChineseVariant,
};
pub use quality::{
    compute_quality_score, content_fingerprint, is_injection, is_near_duplicate,
};
pub use routes::routes;
pub use sentence_splitter::split_sentences;
pub use span_extractor::{
    extract_spans, extract_spans_char_fallback, simple_tokenize, MatchType, TargetSpan, WordToken,
};

#[derive(Debug, Clone, Deserialize)]
pub struct ProcessTextRequest {
    pub text: String,
    pub target_headword: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessedSentence {
    pub text: String,
    pub word_tokens: Vec<WordToken>,
    pub target_spans: Vec<TargetSpan>,
    pub quality_score: f32,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessTextResponse {
    pub normalized_text: String,
    pub sentences: Vec<ProcessedSentence>,
    pub injection_detected: bool,
}

pub fn process_text(request: ProcessTextRequest) -> ProcessTextResponse {
    let normalized_text = normalize(&request.text);
    let target = request
        .target_headword
        .as_deref()
        .map(str::trim)
        .filter(|headword| !headword.is_empty());
    let sentences = split_sentences(&normalized_text)
        .into_iter()
        .map(|sentence| {
            let word_tokens = simple_tokenize(&sentence);
            let target_spans = target
                .map(|headword| extract_spans(&sentence, headword, &word_tokens))
                .unwrap_or_default();
            ProcessedSentence {
                quality_score: compute_quality_score(&sentence),
                fingerprint: content_fingerprint(&sentence),
                text: sentence,
                word_tokens,
                target_spans,
            }
        })
        .collect();

    ProcessTextResponse {
        injection_detected: is_injection(&normalized_text),
        normalized_text,
        sentences,
    }
}

#[cfg(test)]
mod tests;
