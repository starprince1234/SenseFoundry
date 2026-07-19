use sha2::{Digest, Sha256};

pub fn compute_quality_score(sentence: &str) -> f32 {
    let len = sentence.chars().count();
    if len < 5 {
        return 0.1;
    }
    if len > 300 {
        return 0.5;
    }

    let chinese_chars = sentence.chars().filter(|character| is_han(*character)).count();
    let ratio = chinese_chars as f32 / len as f32;
    if ratio < 0.3 {
        0.3
    } else {
        0.8 + ratio * 0.2
    }
}

pub fn is_injection(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("<script") || lower.contains("drop table") || lower.contains("'; --")
}

pub fn content_fingerprint(text: &str) -> String {
    let canonical: String = text
        .chars()
        .filter(|character| !character.is_whitespace() && !character.is_ascii_punctuation())
        .flat_map(char::to_lowercase)
        .collect();
    hex::encode(Sha256::digest(canonical.as_bytes()))
}

pub fn is_near_duplicate(left: &str, right: &str, threshold: f32) -> bool {
    if content_fingerprint(left) == content_fingerprint(right) {
        return true;
    }

    let left_bigrams = bigrams(left);
    let right_bigrams = bigrams(right);
    if left_bigrams.is_empty() || right_bigrams.is_empty() {
        return false;
    }

    let intersection = left_bigrams
        .iter()
        .filter(|gram| right_bigrams.contains(*gram))
        .count();
    let union = left_bigrams.len() + right_bigrams.len() - intersection;
    intersection as f32 / union as f32 >= threshold.clamp(0.0, 1.0)
}

fn bigrams(text: &str) -> Vec<String> {
    let chars: Vec<char> = text
        .chars()
        .filter(|character| !character.is_whitespace() && !character.is_ascii_punctuation())
        .flat_map(char::to_lowercase)
        .collect();

    match chars.as_slice() {
        [] => Vec::new(),
        [character] => vec![character.to_string()],
        _ => chars
            .windows(2)
            .map(|window| window.iter().collect())
            .collect(),
    }
}

fn is_han(character: char) -> bool {
    matches!(
        character,
        '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}' | '\u{f900}'..='\u{faff}'
    )
}
