use chrono::Utc;
use kernel::events::DomainEvent;
use kernel::{AppError, AppResult, EventBus, Page, PageParams};
use sqlx::PgPool;
use uuid::Uuid;

use crate::card::{append_annotation, AnnotationEntry, CardStatus, CorpusCard};

const CARD_SELECT: &str = r#"
    SELECT card.id,
           card.usage_instance_id,
           instance.target_headword,
           instance.sentence_text,
           instance.context_window,
           COALESCE((
               SELECT jsonb_agg(to_jsonb(span) ORDER BY span.start_char, span.id)
               FROM target_spans span
               WHERE span.usage_instance_id = card.usage_instance_id
                 AND span.deleted_at IS NULL
           ), '[]'::jsonb) AS target_spans,
           card.status,
           COALESCE(card.quality_score, 0)::real AS quality_score,
           card.annotation,
           instance.model_version,
           instance.feature_version,
           jsonb_build_object(
               'usage_instance_id', card.usage_instance_id,
               'document_id', instance.document_id,
               'document_version_id', instance.document_version_id,
               'processing_job_id', instance.processing_job_id
           ) AS provenance_chain,
           card.created_at,
           card.updated_at,
           card.deleted_at
    FROM corpus_cards card
    JOIN usage_instances instance ON instance.id = card.usage_instance_id
"#;

#[allow(clippy::too_many_arguments)]
pub async fn create_card_from_instance(
    pool: &PgPool,
    event_bus: &EventBus,
    usage_instance_id: Uuid,
    target_headword: &str,
    sentence_text: &str,
    target_spans: serde_json::Value,
    quality_score: f32,
    model_version: Option<String>,
    feature_version: Option<String>,
) -> Result<CorpusCard, AppError> {
    if !(0.0..=1.0).contains(&quality_score) {
        return Err(AppError::Unprocessable(
            "quality_score must be between 0 and 1".into(),
        ));
    }

    let id = Uuid::new_v4();
    let now = Utc::now();
    let provenance_chain = serde_json::json!({
        "usage_instance_id": usage_instance_id,
        "model_version": model_version,
        "feature_version": feature_version,
    });

    sqlx::query(
        r#"INSERT INTO corpus_cards
           (id, usage_instance_id, status, quality_score, annotation, created_at, updated_at)
           VALUES ($1, $2, $3, $4, '[]'::jsonb, $5, $5)"#,
    )
    .bind(id)
    .bind(usage_instance_id)
    .bind(CardStatus::Processing)
    .bind(f64::from(quality_score))
    .bind(now)
    .execute(pool)
    .await?;

    let card = CorpusCard {
        id,
        usage_instance_id,
        target_headword: target_headword.to_owned(),
        sentence_text: sentence_text.to_owned(),
        context_window: None,
        target_spans,
        status: CardStatus::Processing,
        quality_score,
        annotation: Some(serde_json::json!([])),
        model_version,
        feature_version,
        provenance_chain: Some(provenance_chain),
        created_at: now,
        updated_at: now,
        deleted_at: None,
    };

    event_bus.publish(DomainEvent::CardVerified { card_id: id });
    Ok(card)
}

pub async fn get_card(pool: &PgPool, id: Uuid) -> AppResult<CorpusCard> {
    let query = format!("{CARD_SELECT} WHERE card.id = $1 AND card.deleted_at IS NULL");
    sqlx::query_as(&query)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("corpus card {id}")))
}

pub async fn list_cards(pool: &PgPool, params: &PageParams) -> AppResult<Page<CorpusCard>> {
    let query = format!(
        "{CARD_SELECT} WHERE card.deleted_at IS NULL \
         ORDER BY card.created_at DESC, card.id DESC LIMIT $1 OFFSET $2"
    );
    let items = sqlx::query_as(&query)
        .bind(params.limit())
        .bind(params.offset())
        .fetch_all(pool)
        .await?;
    let total = sqlx::query_scalar("SELECT COUNT(*) FROM corpus_cards WHERE deleted_at IS NULL")
        .fetch_one(pool)
        .await?;
    Ok(Page::new(items, params, Some(total)))
}

pub async fn update_card_status(
    pool: &PgPool,
    id: Uuid,
    next: CardStatus,
    annotated_by: Uuid,
) -> AppResult<CorpusCard> {
    let current = get_card(pool, id).await?;
    if !current.status.can_transition_to(&next) {
        return Err(AppError::Conflict(format!(
            "invalid card status transition from {:?} to {:?}",
            current.status, next
        )));
    }

    let entry = AnnotationEntry {
        annotated_by,
        annotated_at: Utc::now(),
        correction_type: "status_override".into(),
        before: serde_json::json!({ "status": current.status }),
        after: serde_json::json!({ "status": next }),
    };
    let annotation = append_annotation(&current.annotation, entry)?;

    sqlx::query(
        r#"UPDATE corpus_cards
           SET status = $2, annotation = $3, updated_at = NOW()
           WHERE id = $1 AND deleted_at IS NULL"#,
    )
    .bind(id)
    .bind(next)
    .bind(annotation)
    .execute(pool)
    .await?;
    get_card(pool, id).await
}

pub async fn annotate_card(
    pool: &PgPool,
    id: Uuid,
    entry: AnnotationEntry,
) -> AppResult<CorpusCard> {
    let current = get_card(pool, id).await?;
    let annotation = append_annotation(&current.annotation, entry)?;
    sqlx::query(
        r#"UPDATE corpus_cards
           SET annotation = $2, updated_at = NOW()
           WHERE id = $1 AND deleted_at IS NULL"#,
    )
    .bind(id)
    .bind(annotation)
    .execute(pool)
    .await?;
    get_card(pool, id).await
}
