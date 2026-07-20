use chrono::Utc;
use job_queue::EnqueueJob;
use kernel::{AppError, AppResult};
use serde::Serialize;
use sqlx::{PgPool, Row};
use uuid::Uuid;

const CLUSTER_JOB_TYPE: &str = "cluster_headword";

#[derive(Debug, Clone)]
pub struct JobQueue {
    pool: PgPool,
}

impl JobQueue {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn enqueue_clustering(
        &self,
        headword: &str,
        embeddings: Vec<EmbeddingInput>,
        random_seed: i32,
    ) -> AppResult<Uuid> {
        let embedding_ids = embeddings
            .iter()
            .map(|item| item.usage_instance_id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let payload = serde_json::json!({
            "headword": headword,
            "embeddings": embeddings,
            "random_seed": random_seed,
        });
        let job = job_queue::enqueue(
            &self.pool,
            EnqueueJob {
                job_type: CLUSTER_JOB_TYPE.into(),
                payload,
                max_attempts: 3,
                idempotency_key: format!("{CLUSTER_JOB_TYPE}:{headword}:{embedding_ids}"),
                random_seed: Some(random_seed),
                scheduled_at: Utc::now(),
            },
        )
        .await?;

        Ok(job.id)
    }
}

#[derive(Debug, Serialize)]
struct EmbeddingInput {
    usage_instance_id: Uuid,
    embedding: Vec<f32>,
}

pub async fn trigger_clustering(
    pool: &PgPool,
    headword: &str,
    job_queue: &JobQueue,
) -> AppResult<Uuid> {
    let headword = headword.trim();
    if headword.is_empty() {
        return Err(AppError::Unprocessable("headword must not be empty".into()));
    }

    let rows = sqlx::query(
        r#"
        SELECT DISTINCT ui.id, ui.embedding::text AS embedding
        FROM usage_instances ui
        JOIN corpus_cards cc ON cc.usage_instance_id = ui.id
        WHERE ui.target_headword = $1
          AND cc.status = 'CLUSTERED'
          AND ui.embedding IS NOT NULL
          AND ui.deleted_at IS NULL
          AND cc.deleted_at IS NULL
        ORDER BY ui.id
        "#,
    )
    .bind(headword)
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Err(AppError::Unprocessable(format!(
            "no clustered usage embeddings found for headword {headword}"
        )));
    }

    let embeddings = rows
        .into_iter()
        .map(|row| {
            let usage_instance_id = row.try_get("id")?;
            let raw: String = row.try_get("embedding")?;
            let embedding = parse_vector(&raw)?;
            Ok(EmbeddingInput {
                usage_instance_id,
                embedding,
            })
        })
        .collect::<AppResult<Vec<_>>>()?;

    job_queue
        .enqueue_clustering(headword, embeddings, random_seed(headword))
        .await
}

fn random_seed(headword: &str) -> i32 {
    let hash = headword.bytes().fold(2_166_136_261_u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(16_777_619)
    });
    (hash & i32::MAX as u32) as i32
}

fn parse_vector(raw: &str) -> AppResult<Vec<f32>> {
    let values = raw
        .trim()
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("invalid vector representation")))?;

    values
        .split(',')
        .map(|value| {
            value.trim().parse::<f32>().map_err(|error| {
                AppError::Internal(anyhow::anyhow!("invalid vector component: {error}"))
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pgvector_text() {
        assert_eq!(
            parse_vector("[0.25,-1,2.5]").expect("valid pgvector test input"),
            vec![0.25, -1.0, 2.5]
        );
    }

    #[test]
    fn produces_stable_random_seed() {
        assert_eq!(random_seed("打"), random_seed("打"));
        assert_ne!(random_seed("打"), random_seed("做"));
    }
}
