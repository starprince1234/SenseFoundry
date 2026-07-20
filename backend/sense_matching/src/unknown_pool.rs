use chrono::{DateTime, Utc};
use kernel::{AppError, Page, PageParams};
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

/// Add a card to the terminal unknown pool (BR-008).
pub async fn add_to_unknown_pool(
    pool: &PgPool,
    card_id: Uuid,
    reason: &str,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO unknown_pool (id, corpus_card_id, reason, created_at)
        SELECT $1, $2, $3, NOW()
        WHERE NOT EXISTS (
            SELECT 1
            FROM unknown_pool
            WHERE corpus_card_id = $2 AND deleted_at IS NULL
        )
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(card_id)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_unknown_pool(
    pool: &PgPool,
    target_headword: &str,
    params: &PageParams,
) -> Result<Page<UnknownPoolEntry>, AppError> {
    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM unknown_pool up
        JOIN corpus_cards cc ON cc.id = up.corpus_card_id
        JOIN usage_instances ui ON ui.id = cc.usage_instance_id
        WHERE ui.target_headword = $1
          AND up.deleted_at IS NULL
          AND cc.deleted_at IS NULL
          AND ui.deleted_at IS NULL
        "#,
    )
    .bind(target_headword)
    .fetch_one(pool)
    .await?;

    let entries = sqlx::query_as::<_, UnknownPoolEntry>(
        r#"
        SELECT
            up.id,
            up.corpus_card_id,
            COALESCE(up.reason, '') AS reason,
            up.created_at
        FROM unknown_pool up
        JOIN corpus_cards cc ON cc.id = up.corpus_card_id
        JOIN usage_instances ui ON ui.id = cc.usage_instance_id
        WHERE ui.target_headword = $1
          AND up.deleted_at IS NULL
          AND cc.deleted_at IS NULL
          AND ui.deleted_at IS NULL
        ORDER BY up.created_at DESC, up.id
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(target_headword)
    .bind(params.limit())
    .bind(params.offset())
    .fetch_all(pool)
    .await?;

    Ok(Page::new(entries, params, Some(total)))
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct UnknownPoolEntry {
    pub id: Uuid,
    pub corpus_card_id: Uuid,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}
