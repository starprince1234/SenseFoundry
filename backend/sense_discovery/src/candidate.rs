use std::collections::HashMap;

use kernel::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SenseCandidate {
    pub id: Uuid,
    pub cluster_id: Uuid,
    pub headword_id: Uuid,
    pub target_lexeme_id: Option<Uuid>,
    pub proposed_gloss: Option<String>,
    pub status: String,
    pub evidence_count: i32,
}

pub async fn propose_sense_candidate(
    pool: &PgPool,
    cluster_id: Uuid,
) -> AppResult<SenseCandidate> {
    let rows = sqlx::query(
        r#"
        SELECT
            ui.id AS usage_instance_id,
            NULLIF(BTRIM(cc.annotation ->> 'gloss'), '') AS gloss
        FROM cluster_memberships cm
        JOIN corpus_cards cc ON cc.id = cm.corpus_card_id
        JOIN usage_instances ui ON ui.id = cc.usage_instance_id
        WHERE cm.cluster_id = $1
          AND cc.deleted_at IS NULL
          AND ui.deleted_at IS NULL
        ORDER BY cm.probability DESC NULLS LAST, cm.created_at, cm.id
        "#,
    )
    .bind(cluster_id)
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Err(AppError::NotFound(format!(
            "cluster {cluster_id} has no members"
        )));
    }

    let representative_instance_id = rows[0].try_get::<Uuid, _>("usage_instance_id")?;
    let glosses = rows
        .iter()
        .map(|row| row.try_get::<Option<String>, _>("gloss"))
        .collect::<Result<Vec<_>, _>>()?;
    let proposed_gloss = most_common_gloss(glosses.into_iter().flatten());
    let evidence_count = i32::try_from(rows.len())
        .map_err(|error| AppError::Internal(anyhow::Error::new(error)))?;

    let mut transaction = pool.begin().await?;
    let cluster = sqlx::query(
        r#"
        UPDATE clusters
        SET representative_instance_id = COALESCE(representative_instance_id, $2)
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING cluster_run_id
        "#,
    )
    .bind(cluster_id)
    .bind(representative_instance_id)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("cluster {cluster_id} not found")))?;
    let cluster_run_id: Uuid = cluster.try_get("cluster_run_id")?;

    let candidate = sqlx::query_as::<_, SenseCandidate>(
        r#"
        INSERT INTO sense_candidates
            (cluster_id, headword_id, proposed_gloss, status, evidence_count)
        SELECT $1, cr.headword_id, $2, 'proposed', $3
        FROM cluster_runs cr
        WHERE cr.id = $4 AND cr.deleted_at IS NULL
        RETURNING id, cluster_id, headword_id, target_lexeme_id,
                  proposed_gloss, status, evidence_count
        "#,
    )
    .bind(cluster_id)
    .bind(proposed_gloss)
    .bind(evidence_count)
    .bind(cluster_run_id)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("cluster run {cluster_run_id} not found")))?;

    transaction.commit().await?;
    Ok(candidate)
}

fn most_common_gloss(glosses: impl Iterator<Item = String>) -> Option<String> {
    let mut counts = HashMap::<String, usize>::new();
    for gloss in glosses {
        *counts.entry(gloss).or_default() += 1;
    }

    counts
        .into_iter()
        .max_by(|(left_gloss, left_count), (right_gloss, right_count)| {
            left_count
                .cmp(right_count)
                .then_with(|| right_gloss.cmp(left_gloss))
        })
        .map(|(gloss, _)| gloss)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_most_common_gloss_with_deterministic_ties() {
        let glosses = ["strike", "hit", "strike", "hit", "tap"]
            .into_iter()
            .map(str::to_owned);
        assert_eq!(most_common_gloss(glosses).as_deref(), Some("hit"));
    }
}
