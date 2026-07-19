use std::collections::HashSet;
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Example {
    pub id: Uuid,
    pub sentence: String,
    pub quality_score: f32,
    pub corpus_date: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RankedExample {
    #[serde(flatten)]
    pub example: Example,
    pub diversity: f32,
    pub recency: f32,
    pub score: f32,
}

#[derive(Clone, Default)]
pub struct ExampleRankingService {
    ranked: Arc<RwLock<Vec<RankedExample>>>,
}

impl ExampleRankingService {
    pub fn rank(&self, mut examples: Vec<Example>) -> Vec<RankedExample> {
        examples.sort_by(|left, right| {
            right
                .quality_score
                .total_cmp(&left.quality_score)
                .then_with(|| left.id.cmp(&right.id))
        });
        let now = Utc::now();
        let mut selected_sentences: Vec<String> = Vec::new();
        let mut ranked = Vec::with_capacity(examples.len());
        for example in examples {
            let max_overlap = selected_sentences
                .iter()
                .map(|sentence| ngram_overlap(&example.sentence, sentence))
                .fold(0.0_f32, f32::max);
            let diversity = 1.0 - max_overlap;
            let age_days = (now - example.corpus_date).num_days().max(0) as f32;
            let recency = 1.0 / (1.0 + age_days / 365.0);
            let score = example.quality_score * 0.4 + diversity * 0.3 + recency * 0.3;
            selected_sentences.push(example.sentence.clone());
            ranked.push(RankedExample {
                example,
                diversity,
                recency,
                score,
            });
        }
        ranked.sort_by(|left, right| right.score.total_cmp(&left.score));
        *self.ranked.write().unwrap_or_else(|error| error.into_inner()) = ranked.clone();
        ranked
    }

    pub fn list(&self) -> Vec<RankedExample> {
        self.ranked
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }
}

fn ngram_overlap(left: &str, right: &str) -> f32 {
    let left = char_ngrams(left, 3);
    let right = char_ngrams(right, 3);
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    let intersection = left.intersection(&right).count() as f32;
    let union = left.union(&right).count() as f32;
    intersection / union
}

fn char_ngrams(value: &str, size: usize) -> HashSet<String> {
    let chars: Vec<_> = value.to_lowercase().chars().collect();
    chars
        .windows(size)
        .map(|window| window.iter().collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    #[test]
    fn penalizes_lexically_similar_sentences() {
        let now = Utc::now();
        let service = ExampleRankingService::default();
        let ranked = service.rank(vec![
            Example {
                id: Uuid::new_v4(),
                sentence: "The river bank was green".into(),
                quality_score: 1.0,
                corpus_date: now,
            },
            Example {
                id: Uuid::new_v4(),
                sentence: "The river bank is green".into(),
                quality_score: 0.9,
                corpus_date: now - Duration::days(1),
            },
        ]);
        assert!(ranked.iter().any(|example| example.diversity < 0.5));
    }
}
