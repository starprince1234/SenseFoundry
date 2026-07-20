use kernel::AppError;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

/// Find nearest neighbors using the pgvector HNSW index.
/// Returns usage instance IDs and their cosine distances.
pub async fn hnsw_search(
    pool: &PgPool,
    query_vector: &[f32],
    target_headword: &str,
    top_k: i64,
) -> Result<Vec<VectorSearchResult>, AppError> {
    let vector = serde_json::to_string(query_vector)
        .map_err(|error| AppError::Internal(error.into()))?;
    let results = sqlx::query_as::<_, VectorSearchResult>(
        r#"
        SELECT
            id,
            (embedding <=> $1::vector) AS distance
        FROM usage_instances
        WHERE target_headword = $2
          AND deleted_at IS NULL
          AND embedding IS NOT NULL
        ORDER BY embedding <=> $1::vector
        LIMIT $3
        "#,
    )
    .bind(vector)
    .bind(target_headword)
    .bind(top_k)
    .fetch_all(pool)
    .await?;
    Ok(results)
}

#[derive(Debug, Clone, FromRow)]
pub struct VectorSearchResult {
    pub id: Uuid,
    pub distance: f64,
}
