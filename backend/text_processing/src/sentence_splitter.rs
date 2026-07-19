use std::sync::OnceLock;

use regex::Regex;

/// Split text into sentences at Chinese punctuation boundaries.
pub fn split_sentences(text: &str) -> Vec<String> {
    static BOUNDARY: OnceLock<Regex> = OnceLock::new();
    let boundary = BOUNDARY.get_or_init(|| {
        Regex::new(r"[。！？；!?;]+")
            .expect("the static sentence-boundary regex must be valid")
    });
    let mut sentences = Vec::new();
    let mut last = 0;

    for matched in boundary.find_iter(text) {
        let end = matched.end();
        let sentence = text[last..end].trim();
        if !sentence.is_empty() {
            sentences.push(sentence.to_string());
        }
        last = end;
    }

    if last < text.len() {
        let sentence = text[last..].trim();
        if !sentence.is_empty() {
            sentences.push(sentence.to_string());
        }
    }

    sentences
}
