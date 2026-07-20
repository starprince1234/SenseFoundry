use kernel::{AppError, AppResult, Page, PageParams};
use regex::Regex;
use sqlx::{PgPool, Postgres, Transaction};
use url::Url;
use uuid::Uuid;

use crate::{CopyrightStatus, Source, SourceKind};

#[derive(Debug, Clone)]
pub struct NewSource {
    pub uri: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub doi: Option<String>,
    pub license: Option<String>,
    pub source_kind: SourceKind,
}

#[derive(Debug, Clone, Default)]
pub struct VerificationEvidence {
    pub isbn: Option<String>,
    pub doi: Option<String>,
    pub license: Option<String>,
    pub url_accessible: bool,
}

pub fn validate_isbn(isbn: &str) -> bool {
    let cleaned: String = isbn
        .chars()
        .filter(|character| character.is_numeric() || *character == 'X')
        .collect();
    cleaned.len() == 10 || cleaned.len() == 13
}

pub fn validate_doi(doi: &str) -> bool {
    Regex::new(r"^10\.")
        .is_ok_and(|pattern| pattern.is_match(doi.trim()))
}

pub async fn create(pool: &PgPool, input: NewSource) -> AppResult<Source> {
    validate_uri(&input.uri)?;
    validate_identifiers(input.isbn.as_deref(), input.doi.as_deref())?;

    let has_identifier = input.isbn.is_some() || input.doi.is_some();
    let status = CopyrightStatus::determine(
        input.license.is_some(),
        has_identifier,
        input.source_kind.as_str(),
        false,
    );
    let (is_storable, is_trainable, is_publishable) = status.authorization_flags();
    let mut transaction = pool.begin().await?;
    let source_id = insert_source(&mut transaction, &input, status, is_storable, is_trainable, is_publishable).await?;

    sqlx::query(
        "INSERT INTO bibliographic_records (source_id, isbn, doi) VALUES ($1, $2, $3)",
    )
    .bind(source_id)
    .bind(&input.isbn)
    .bind(&input.doi)
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;
    get(pool, source_id).await
}

async fn insert_source(
    transaction: &mut Transaction<'_, Postgres>,
    input: &NewSource,
    status: CopyrightStatus,
    is_storable: bool,
    is_trainable: bool,
    is_publishable: bool,
) -> AppResult<Uuid> {
    sqlx::query_scalar(
        r#"INSERT INTO sources
           (uri, title, author, license, source_kind, copyright_status,
            is_storable, is_trainable, is_publishable)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
           RETURNING id"#,
    )
    .bind(&input.uri)
    .bind(&input.title)
    .bind(&input.author)
    .bind(&input.license)
    .bind(input.source_kind)
    .bind(status)
    .bind(is_storable)
    .bind(is_trainable)
    .bind(is_publishable)
    .fetch_one(&mut **transaction)
    .await
    .map_err(AppError::from)
}

pub async fn get(pool: &PgPool, id: Uuid) -> AppResult<Source> {
    source_query()
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("source {id}")))
}

pub async fn list(pool: &PgPool, params: &PageParams) -> AppResult<Page<Source>> {
    let items = sqlx::query_as::<_, Source>(
        r#"SELECT s.id, s.uri, s.title, s.author, b.isbn, b.doi, s.license,
                  s.copyright_status, s.is_storable, s.is_trainable,
                  s.is_publishable, s.source_kind, s.created_at, s.updated_at
           FROM sources s
           LEFT JOIN bibliographic_records b ON b.source_id = s.id AND b.deleted_at IS NULL
           WHERE s.deleted_at IS NULL
           ORDER BY s.created_at DESC
           LIMIT $1 OFFSET $2"#,
    )
    .bind(params.limit())
    .bind(params.offset())
    .fetch_all(pool)
    .await?;
    let total = sqlx::query_scalar("SELECT COUNT(*) FROM sources WHERE deleted_at IS NULL")
        .fetch_one(pool)
        .await?;

    Ok(Page::new(items, params, Some(total)))
}

pub async fn verify(
    pool: &PgPool,
    id: Uuid,
    evidence: VerificationEvidence,
) -> AppResult<Source> {
    validate_identifiers(evidence.isbn.as_deref(), evidence.doi.as_deref())?;
    let current = get(pool, id).await?;
    let isbn = evidence.isbn.or(current.isbn);
    let doi = evidence.doi.or(current.doi);
    let license = evidence.license.or(current.license);
    let status = CopyrightStatus::determine(
        license.is_some(),
        isbn.is_some() || doi.is_some(),
        current.source_kind.as_str(),
        evidence.url_accessible,
    );
    let (is_storable, is_trainable, is_publishable) = status.authorization_flags();
    let mut transaction = pool.begin().await?;

    sqlx::query(
        r#"UPDATE sources
           SET license = $2, copyright_status = $3, is_storable = $4,
               is_trainable = $5, is_publishable = $6, updated_at = NOW()
           WHERE id = $1 AND deleted_at IS NULL"#,
    )
    .bind(id)
    .bind(&license)
    .bind(status)
    .bind(is_storable)
    .bind(is_trainable)
    .bind(is_publishable)
    .execute(&mut *transaction)
    .await?;

    sqlx::query(
        r#"UPDATE bibliographic_records
           SET isbn = $2, doi = $3
           WHERE source_id = $1 AND deleted_at IS NULL"#,
    )
    .bind(id)
    .bind(&isbn)
    .bind(&doi)
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;
    get(pool, id).await
}

fn source_query() -> sqlx::query::QueryAs<'static, Postgres, Source, sqlx::postgres::PgArguments> {
    sqlx::query_as(
        r#"SELECT s.id, s.uri, s.title, s.author, b.isbn, b.doi, s.license,
                  s.copyright_status, s.is_storable, s.is_trainable,
                  s.is_publishable, s.source_kind, s.created_at, s.updated_at
           FROM sources s
           LEFT JOIN bibliographic_records b ON b.source_id = s.id AND b.deleted_at IS NULL
           WHERE s.id = $1 AND s.deleted_at IS NULL"#,
    )
}

fn validate_uri(uri: &str) -> AppResult<()> {
    Url::parse(uri)
        .map(|_| ())
        .map_err(|_| AppError::Unprocessable("source URI must be an absolute URL".into()))
}

fn validate_identifiers(isbn: Option<&str>, doi: Option<&str>) -> AppResult<()> {
    if isbn.is_some_and(|value| !validate_isbn(value)) {
        return Err(AppError::Unprocessable("invalid ISBN".into()));
    }
    if doi.is_some_and(|value| !validate_doi(value)) {
        return Err(AppError::Unprocessable("invalid DOI".into()));
    }
    Ok(())
}
