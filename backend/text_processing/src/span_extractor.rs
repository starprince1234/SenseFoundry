use serde::{Deserialize, Serialize};

/// Match type for target span (3-level representation section 18.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchType {
    /// Single character exact match: 打 in "他打了我".
    CharExact,
    /// Single character embedded in multi-char word: 打 in "打电话".
    CharInLexeme,
    /// Multi-character lexeme match: 打电话 as whole word.
    LexemeMultiChar,
    /// Character at word boundary.
    CharAtBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetSpan {
    pub start_char: usize,
    pub end_char: usize,
    pub surface: String,
    pub target_headword: String,
    pub target_lexeme: Option<String>,
    pub match_type: MatchType,
    pub detector: String,
    pub human_verified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordToken {
    pub text: String,
    pub start_char: usize,
    pub end_char: usize,
    pub pos: Option<String>,
}

/// Character tokenizer used when a segmentation service is unavailable.
pub fn simple_tokenize(sentence: &str) -> Vec<WordToken> {
    sentence
        .chars()
        .enumerate()
        .map(|(index, character)| WordToken {
            text: character.to_string(),
            start_char: index,
            end_char: index + 1,
            pos: None,
        })
        .collect()
}

/// Extract all target spans, preserving both character and containing-lexeme matches.
pub fn extract_spans(
    sentence: &str,
    target_headword: &str,
    word_tokens: &[WordToken],
) -> Vec<TargetSpan> {
    let chars: Vec<char> = sentence.chars().collect();
    let target_chars: Vec<char> = target_headword.chars().collect();
    let target_len = target_chars.len();
    let mut spans = Vec::new();

    if target_len == 0 {
        return spans;
    }

    for index in 0..chars.len() {
        if index + target_len > chars.len()
            || chars[index..index + target_len] != target_chars[..]
        {
            continue;
        }

        let char_span = TargetSpan {
            start_char: index,
            end_char: index + target_len,
            surface: target_headword.to_string(),
            target_headword: target_headword.to_string(),
            target_lexeme: None,
            match_type: MatchType::CharExact,
            detector: "char_scan".into(),
            human_verified: false,
        };

        let containing_word = word_tokens.iter().find(|token| {
            token.start_char <= index
                && token.end_char >= index + target_len
                && token.end_char.saturating_sub(token.start_char) > target_len
        });

        if let Some(word) = containing_word {
            let mut char_in_lexeme = char_span;
            char_in_lexeme.match_type = MatchType::CharInLexeme;
            char_in_lexeme.target_lexeme = Some(word.text.clone());
            spans.push(char_in_lexeme);
            spans.push(TargetSpan {
                start_char: word.start_char,
                end_char: word.end_char,
                surface: word.text.clone(),
                target_headword: target_headword.to_string(),
                target_lexeme: Some(word.text.clone()),
                match_type: MatchType::LexemeMultiChar,
                detector: "word_token".into(),
                human_verified: false,
            });
        } else {
            spans.push(char_span);
        }
    }

    spans
}

/// Extract character matches when word segmentation fails.
pub fn extract_spans_char_fallback(
    sentence: &str,
    target_headword: &str,
    window_size: usize,
) -> Vec<TargetSpan> {
    let chars: Vec<char> = sentence.chars().collect();
    let target_chars: Vec<char> = target_headword.chars().collect();
    let target_len = target_chars.len();

    if target_len == 0 {
        return Vec::new();
    }

    (0..chars.len())
        .filter(|index| {
            *index + target_len <= chars.len()
                && chars[*index..*index + target_len] == target_chars[..]
        })
        .map(|index| {
            let _window_start = index.saturating_sub(window_size);
            let _window_end = (index + target_len + window_size).min(chars.len());
            TargetSpan {
                start_char: index,
                end_char: index + target_len,
                surface: target_headword.to_string(),
                target_headword: target_headword.to_string(),
                target_lexeme: None,
                match_type: MatchType::CharExact,
                detector: "char_fallback".into(),
                human_verified: false,
            }
        })
        .collect()
}
