use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CorpusExample {
    pub id: Uuid,
    pub headword: String,
    pub sense_candidate_id: Uuid,
    pub sentence: String,
    pub corpus_date: NaiveDate,
}

#[derive(Clone, Debug, Serialize)]
pub struct EarliestCorpusExample {
    pub sense_candidate_id: Uuid,
    pub example: CorpusExample,
}

#[derive(Clone, Debug, Serialize)]
pub struct PeriodGroup {
    pub period: String,
    pub examples: Vec<CorpusExample>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DiachronicAnalysis {
    pub headword: String,
    pub scope_note: &'static str,
    pub earliest_by_sense: Vec<EarliestCorpusExample>,
    pub periods: Vec<PeriodGroup>,
}

#[derive(Clone, Default)]
pub struct AnalysisService {
    examples: Arc<RwLock<Vec<CorpusExample>>>,
}

impl AnalysisService {
    pub fn new(examples: Vec<CorpusExample>) -> Self {
        Self {
            examples: Arc::new(RwLock::new(examples)),
        }
    }

    pub fn analyze(&self, headword: &str) -> DiachronicAnalysis {
        let examples = self.examples.read().unwrap_or_else(|error| error.into_inner());
        let mut relevant: Vec<_> = examples
            .iter()
            .filter(|example| example.headword == headword)
            .cloned()
            .collect();
        relevant.sort_by_key(|example| (example.corpus_date, example.id));

        let mut earliest = BTreeMap::new();
        let mut periods: BTreeMap<i32, Vec<CorpusExample>> = BTreeMap::new();
        for example in relevant {
            earliest
                .entry(example.sense_candidate_id)
                .or_insert_with(|| example.clone());
            let decade = example.corpus_date.year().div_euclid(10) * 10;
            periods.entry(decade).or_default().push(example);
        }

        DiachronicAnalysis {
            headword: headword.to_owned(),
            scope_note: "Dates identify the earliest example in the current corpus only; they are not claims about language history.",
            earliest_by_sense: earliest
                .into_iter()
                .map(|(sense_candidate_id, example)| EarliestCorpusExample {
                    sense_candidate_id,
                    example,
                })
                .collect(),
            periods: periods
                .into_iter()
                .map(|(decade, examples)| PeriodGroup {
                    period: format!("{decade}s"),
                    examples,
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_earliest_current_corpus_example_and_groups_decades() {
        let sense = Uuid::new_v4();
        let examples = vec![
            CorpusExample {
                id: Uuid::new_v4(),
                headword: "bank".into(),
                sense_candidate_id: sense,
                sentence: "later".into(),
                corpus_date: NaiveDate::from_ymd_opt(1998, 1, 1).expect("valid date"),
            },
            CorpusExample {
                id: Uuid::new_v4(),
                headword: "bank".into(),
                sense_candidate_id: sense,
                sentence: "earlier".into(),
                corpus_date: NaiveDate::from_ymd_opt(1982, 1, 1).expect("valid date"),
            },
        ];

        let result = AnalysisService::new(examples).analyze("bank");

        assert_eq!(result.earliest_by_sense[0].example.sentence, "earlier");
        assert_eq!(result.periods.len(), 2);
        assert!(result.scope_note.contains("current corpus only"));
    }
}
