use chrono::{DateTime, Utc};
use kernel::{AppError, AppResult, Page, PageParams};
use sha2::{Digest, Sha256};
use sqlx::{postgres::PgRow, PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

use crate::{
    validator::{validate_submission, ValidationError},
    CreateSubmissionRequest, Submission, SubmissionStatus,
};

#[derive(Debug)]
pub enum CreateSubmissionOutcome {
    Created(Submission),
    Existing(Submission),
}

#[derive(Clone)]
pub struct SubmissionService {
    pool: PgPool,
}

impl SubmissionService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        submitter_id: Uuid,
        idempotency_key: &str,
        request: &CreateSubmissionRequest,
    ) -> AppResult<CreateSubmissionOutcome> {
        if idempotency_key.trim().is_empty() {
            return Err(AppError::Unprocessable(
                "Idempotency-Key header must not be empty".into(),
            ));
        }

        if let Some(existing) = self.find_by_idempotency_key(idempotency_key).await? {
            return Ok(CreateSubmissionOutcome::Existing(existing));
        }

        let validation_error = validate_submission(request).err();
        let mut transaction = self.pool.begin().await?;
        let document_id = ensure_document(&mut transaction, request, validation_error.is_some()).await?;
        let id = Uuid::new_v4();
        let status = if validation_error.is_some() {
            SubmissionStatus::Rejected
        } else {
            SubmissionStatus::Pending
        };
        let rejection_reason = validation_error.as_ref().map(ToString::to_string);

        let inserted = sqlx::query(
            "INSERT INTO corpus_submissions \
             (id, submitter_id, document_id, submission_type, status, idempotency_key, \
              rejection_reason, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW()) \
             ON CONFLICT (idempotency_key) DO NOTHING \
             RETURNING id, submitter_id, status, idempotency_key, created_at",
        )
        .bind(id)
        .bind(submitter_id)
        .bind(document_id)
        .bind(&request.submission_type)
        .bind(status)
        .bind(idempotency_key)
        .bind(&rejection_reason)
        .fetch_optional(&mut *transaction)
        .await?;

        let outcome = if let Some(row) = inserted {
            CreateSubmissionOutcome::Created(submission_from_row(row)?)
        } else {
            let row = sqlx::query(
                "SELECT id, submitter_id, status, idempotency_key, created_at \
                 FROM corpus_submissions WHERE idempotency_key = $1",
            )
            .bind(idempotency_key)
            .fetch_one(&mut *transaction)
            .await?;
            CreateSubmissionOutcome::Existing(submission_from_row(row)?)
        };
        transaction.commit().await?;

        if let Some(error) = validation_error {
            tracing::warn!(
                %submitter_id,
                idempotency_key,
                reason = %error,
                "rejected corpus submission"
            );
            return Err(validation_app_error(error));
        }

        Ok(outcome)
    }

    pub async fn list(&self, params: &PageParams) -> AppResult<Page<Submission>> {
        let rows = sqlx::query(
            "SELECT id, submitter_id, status, idempotency_key, created_at \
             FROM corpus_submissions WHERE deleted_at IS NULL \
             ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(params.limit())
        .bind(params.offset())
        .fetch_all(&self.pool)
        .await?;
        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM corpus_submissions WHERE deleted_at IS NULL",
        )
        .fetch_one(&self.pool)
        .await?;
        let submissions = rows
            .into_iter()
            .map(submission_from_row)
            .collect::<AppResult<Vec<_>>>()?;

        Ok(Page::new(submissions, params, Some(total)))
    }

    pub async fn get(&self, id: Uuid) -> AppResult<Submission> {
        let row = sqlx::query(
            "SELECT id, submitter_id, status, idempotency_key, created_at \
             FROM corpus_submissions WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("submission {id}")))?;

        submission_from_row(row)
    }

    async fn find_by_idempotency_key(&self, key: &str) -> AppResult<Option<Submission>> {
        let row = sqlx::query(
            "SELECT id, submitter_id, status, idempotency_key, created_at \
             FROM corpus_submissions WHERE idempotency_key = $1",
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        row.map(submission_from_row).transpose()
    }
}

async fn ensure_document(
    transaction: &mut Transaction<'_, Postgres>,
    request: &CreateSubmissionRequest,
    rejected: bool,
) -> AppResult<Uuid> {
    let content = request
        .text
        .as_deref()
        .or(request.source_url.as_deref())
        .unwrap_or_default();
    let content_hash = hex::encode(Sha256::digest(content.as_bytes()));

    if let Some(id) = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM documents WHERE content_hash = $1 AND deleted_at IS NULL",
    )
    .bind(&content_hash)
    .fetch_optional(&mut **transaction)
    .await?
    {
        return Ok(id);
    }

    let source_id = Uuid::new_v4();
    let source_uri = request
        .source_url
        .clone()
        .unwrap_or_else(|| format!("user-submission://{source_id}"));
    sqlx::query(
        "INSERT INTO sources \
         (id, uri, source_kind, copyright_status, is_authoritative, created_at) \
         VALUES ($1, $2, 'user_submission', 'user_submitted', FALSE, NOW())",
    )
    .bind(source_id)
    .bind(source_uri)
    .execute(&mut **transaction)
    .await?;

    let document_id = Uuid::new_v4();
    let original_bytes = (!rejected)
        .then(|| request.text.as_ref().map(|text| text.as_bytes().to_vec()))
        .flatten();
    let byte_count = original_bytes.as_ref().map(Vec::len).unwrap_or_default() as i32;
    let row = sqlx::query(
        "INSERT INTO documents \
         (id, source_id, content_hash, original_bytes, encoding, byte_count, created_at) \
         VALUES ($1, $2, $3, $4, 'utf-8', $5, NOW()) \
         ON CONFLICT (content_hash) DO UPDATE SET content_hash = EXCLUDED.content_hash \
         RETURNING id",
    )
    .bind(document_id)
    .bind(source_id)
    .bind(content_hash)
    .bind(original_bytes)
    .bind(byte_count)
    .fetch_one(&mut **transaction)
    .await?;

    Ok(row.try_get("id")?)
}

fn submission_from_row(row: PgRow) -> AppResult<Submission> {
    Ok(Submission {
        id: row.try_get("id")?,
        submitter_id: row.try_get("submitter_id")?,
        status: row.try_get("status")?,
        idempotency_key: row.try_get("idempotency_key")?,
        created_at: row.try_get::<DateTime<Utc>, _>("created_at")?,
    })
}

fn validation_app_error(error: ValidationError) -> AppError {
    AppError::Unprocessable(error.to_string())
}
