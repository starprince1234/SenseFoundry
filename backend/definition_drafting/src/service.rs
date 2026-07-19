use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EvidenceCard {
    pub id: Uuid,
    pub sense_candidate_id: Uuid,
    pub sentence_text: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct DefinitionDraft {
    pub id: Uuid,
    pub sense_candidate_id: Uuid,
    pub headword: String,
    pub pos: String,
    pub definition: String,
    pub evidence_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum DraftError {
    #[error("sense candidate has no evidence cards")]
    MissingEvidence,
    #[error("definition draft not found")]
    NotFound,
    #[error("LLM gateway failed: {0}")]
    Gateway(String),
}

pub trait LlmGateway: Send + Sync {
    fn draft(
        &self,
        headword: &str,
        pos: &str,
        evidence: &[EvidenceCard],
    ) -> Result<String, DraftError>;
}

#[derive(Clone)]
pub struct DraftService {
    gateway: Arc<dyn LlmGateway>,
    cards: Arc<RwLock<Vec<EvidenceCard>>>,
    drafts: Arc<RwLock<HashMap<Uuid, DefinitionDraft>>>,
}

impl DraftService {
    pub fn new(gateway: Arc<dyn LlmGateway>, cards: Vec<EvidenceCard>) -> Self {
        Self {
            gateway,
            cards: Arc::new(RwLock::new(cards)),
            drafts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create(
        &self,
        sense_candidate_id: Uuid,
        headword: &str,
        pos: &str,
    ) -> Result<DefinitionDraft, DraftError> {
        let cards = self.cards.read().unwrap_or_else(|error| error.into_inner());
        let evidence: Vec<_> = cards
            .iter()
            .filter(|card| card.sense_candidate_id == sense_candidate_id)
            .cloned()
            .collect();
        if evidence.is_empty() {
            return Err(DraftError::MissingEvidence);
        }
        let definition = self.gateway.draft(headword, pos, &evidence)?;
        let draft = DefinitionDraft {
            id: Uuid::new_v4(),
            sense_candidate_id,
            headword: headword.to_owned(),
            pos: pos.to_owned(),
            definition,
            evidence_ids: evidence.iter().map(|card| card.id).collect(),
            created_at: Utc::now(),
        };
        self.drafts
            .write()
            .unwrap_or_else(|error| error.into_inner())
            .insert(draft.id, draft.clone());
        Ok(draft)
    }

    pub fn get(&self, id: Uuid) -> Result<DefinitionDraft, DraftError> {
        self.drafts
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .get(&id)
            .cloned()
            .ok_or(DraftError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Gateway;

    impl LlmGateway for Gateway {
        fn draft(
            &self,
            _headword: &str,
            _pos: &str,
            evidence: &[EvidenceCard],
        ) -> Result<String, DraftError> {
            Ok(format!("definition from {} cards", evidence.len()))
        }
    }

    #[test]
    fn records_all_supporting_corpus_card_ids() {
        let candidate = Uuid::new_v4();
        let cards = vec![EvidenceCard {
            id: Uuid::new_v4(),
            sense_candidate_id: candidate,
            sentence_text: "evidence".into(),
        }];
        let service = DraftService::new(Arc::new(Gateway), cards.clone());
        let draft = service
            .create(candidate, "bank", "noun")
            .expect("draft should be created");
        assert_eq!(draft.evidence_ids, vec![cards[0].id]);
        assert_eq!(service.get(draft.id).expect("stored draft").id, draft.id);
    }
}
