use kernel::AppError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

pub const RERANK_THRESHOLD: f32 = 0.5;
const RECALL_LIMIT: i64 = 20;

pub fn detect_unknown(rerank_scores: &[(Uuid, f32)]) -> bool {
    rerank_scores.is_empty()
        || rerank_scores
            .iter()
            .all(|(_, score)| *score < RERANK_THRESHOLD)
}

#[derive(Clone)]
pub struct SenseMatcher {
    pool: PgPool,
    inference_url: String,
    client: Client,
}

impl SenseMatcher {
    pub fn new(pool: PgPool, inference_url: impl Into<String>) -> Self {
        Self {
            pool,
            inference_url: inference_url.into().trim_end_matches('/').to_owned(),
            client: Client::new(),
        }
    }

    pub async fn match_card(
        &self,
        card_id: Uuid,
        target_headword: &str,
        h_target: &[f32],
    ) -> Result<MatchResult, AppError> {
        let instance_text = sqlx::query_scalar::<_, String>(
            r#"
            SELECT ui.sentence_text
            FROM corpus_cards cc
            JOIN usage_instances ui ON ui.id = cc.usage_instance_id
            WHERE cc.id = $1
              AND cc.deleted_at IS NULL
              AND ui.deleted_at IS NULL
            "#,
        )
        .bind(card_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("corpus card {card_id}")))?;

        let vector = serde_json::to_string(h_target)
            .map_err(|error| AppError::Internal(error.into()))?;
        let recall_candidates = sqlx::query_as::<_, RecallCandidate>(
            r#"
            SELECT
                rs.id,
                (rs.embedding <=> $1::vector) AS distance,
                rs.gloss
            FROM reference_senses rs
            WHERE rs.headword_id = (
                    SELECT id
                    FROM headwords
                    WHERE character = $2 AND deleted_at IS NULL
                    LIMIT 1
                )
              AND rs.deleted_at IS NULL
              AND rs.embedding IS NOT NULL
            ORDER BY rs.embedding <=> $1::vector
            LIMIT $3
            "#,
        )
        .bind(vector)
        .bind(target_headword)
        .bind(RECALL_LIMIT)
        .fetch_all(&self.pool)
        .await?;

        if recall_candidates.is_empty() {
            return Ok(MatchResult::unknown(card_id));
        }

        let rerank_request = RerankRequest {
            items: recall_candidates
                .iter()
                .map(|candidate| RerankItem {
                    instance_text: &instance_text,
                    sense_gloss: &candidate.gloss,
                    reference_sense_id: candidate.id,
                })
                .collect(),
        };
        let rerank_response = self
            .client
            .post(format!("{}/rerank", self.inference_url))
            .json(&rerank_request)
            .send()
            .await
            .map_err(|error| AppError::Internal(error.into()))?
            .error_for_status()
            .map_err(|error| AppError::Internal(error.into()))?
            .json::<RerankResponse>()
            .await
            .map_err(|error| AppError::Internal(error.into()))?;

        let scores: Vec<_> = rerank_response
            .scores
            .into_iter()
            .filter(|score| score.score.is_finite())
            .map(|score| (score.reference_sense_id, score.score))
            .collect();
        if detect_unknown(&scores) {
            return Ok(MatchResult::unknown(card_id));
        }

        let (matched_sense_id, rerank_score) = scores
            .iter()
            .max_by(|left, right| left.1.total_cmp(&right.1))
            .copied()
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("rerank scores were empty")))?;
        let match_score = recall_candidates
            .iter()
            .find(|candidate| candidate.id == matched_sense_id)
            .map(|candidate| candidate.distance as f32);

        Ok(MatchResult {
            card_id,
            matched_sense_id: Some(matched_sense_id),
            match_score,
            rerank_score: Some(rerank_score),
            is_unknown: false,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchResult {
    pub card_id: Uuid,
    pub matched_sense_id: Option<Uuid>,
    pub match_score: Option<f32>,
    pub rerank_score: Option<f32>,
    pub is_unknown: bool,
}

impl MatchResult {
    fn unknown(card_id: Uuid) -> Self {
        Self {
            card_id,
            matched_sense_id: None,
            match_score: None,
            rerank_score: None,
            is_unknown: true,
        }
    }
}

#[derive(Debug, FromRow)]
struct RecallCandidate {
    id: Uuid,
    distance: f64,
    gloss: String,
}

#[derive(Serialize)]
struct RerankRequest<'a> {
    items: Vec<RerankItem<'a>>,
}

#[derive(Serialize)]
struct RerankItem<'a> {
    instance_text: &'a str,
    sense_gloss: &'a str,
    reference_sense_id: Uuid,
}

#[derive(Deserialize)]
struct RerankResponse {
    #[serde(default)]
    scores: Vec<RerankScore>,
}

#[derive(Deserialize)]
struct RerankScore {
    reference_sense_id: Uuid,
    score: f32,
}
