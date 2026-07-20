use kernel::{AppError, AppResult, Page, PageParams};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{ModelType, ModelVersion, PromptTemplate, INITIAL_FEATURE_VERSION};

#[derive(Debug, Deserialize)]
pub struct CreateModel {
    pub name: String,
    pub revision: String,
    pub model_type: ModelType,
    pub feature_version: Option<String>,
    #[serde(default = "default_active")]
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateModel {
    pub revision: Option<String>,
    pub feature_version: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePromptTemplate {
    pub name: String,
    pub template_text: String,
    pub version: String,
    pub model_type: ModelType,
    #[serde(default = "default_active")]
    pub is_active: bool,
}

const fn default_active() -> bool {
    true
}

pub async fn create_model(pool: &PgPool, input: CreateModel) -> AppResult<ModelVersion> {
    validate_required("model name", &input.name)?;
    validate_required("model revision", &input.revision)?;
    let feature_version = input
        .feature_version
        .unwrap_or_else(|| INITIAL_FEATURE_VERSION.to_owned());
    validate_required("feature version", &feature_version)?;
    let model_version = format!("{}@{}", input.name, input.revision);

    sqlx::query_as(
        r#"INSERT INTO model_registry
           (model_name, model_version, revision, model_type, feature_version, is_active)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, model_name AS name, revision, model_type,
                     feature_version, is_active, created_at"#,
    )
    .bind(input.name)
    .bind(model_version)
    .bind(input.revision)
    .bind(input.model_type)
    .bind(feature_version)
    .bind(input.is_active)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

pub async fn get_model(pool: &PgPool, id: Uuid) -> AppResult<ModelVersion> {
    sqlx::query_as(
        r#"SELECT id, model_name AS name, revision, model_type,
                  feature_version, is_active, created_at
           FROM model_registry
           WHERE id = $1 AND deleted_at IS NULL"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("model version {id}")))
}

pub async fn list_models(pool: &PgPool, params: &PageParams) -> AppResult<Page<ModelVersion>> {
    let items = sqlx::query_as(
        r#"SELECT id, model_name AS name, revision, model_type,
                  feature_version, is_active, created_at
           FROM model_registry
           WHERE deleted_at IS NULL
           ORDER BY created_at DESC, id DESC
           LIMIT $1 OFFSET $2"#,
    )
    .bind(params.limit())
    .bind(params.offset())
    .fetch_all(pool)
    .await?;
    let total = sqlx::query_scalar("SELECT COUNT(*) FROM model_registry WHERE deleted_at IS NULL")
        .fetch_one(pool)
        .await?;
    Ok(Page::new(items, params, Some(total)))
}

pub async fn update_model(
    pool: &PgPool,
    id: Uuid,
    input: UpdateModel,
) -> AppResult<ModelVersion> {
    if let Some(revision) = input.revision.as_deref() {
        validate_required("model revision", revision)?;
    }
    if let Some(feature_version) = input.feature_version.as_deref() {
        validate_required("feature version", feature_version)?;
    }

    sqlx::query_as(
        r#"UPDATE model_registry
           SET revision = COALESCE($2, revision),
               model_version = model_name || '@' || COALESCE($2, revision),
               feature_version = COALESCE($3, feature_version),
               is_active = COALESCE($4, is_active)
           WHERE id = $1 AND deleted_at IS NULL
           RETURNING id, model_name AS name, revision, model_type,
                     feature_version, is_active, created_at"#,
    )
    .bind(id)
    .bind(input.revision)
    .bind(input.feature_version)
    .bind(input.is_active)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("model version {id}")))
}

pub async fn create_template(
    pool: &PgPool,
    input: CreatePromptTemplate,
) -> AppResult<PromptTemplate> {
    validate_required("template name", &input.name)?;
    validate_required("template text", &input.template_text)?;
    validate_required("template version", &input.version)?;

    sqlx::query_as(
        r#"INSERT INTO prompt_templates
           (name, template_text, version, model_type, is_active)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, name, template_text, version, model_type, is_active, created_at"#,
    )
    .bind(input.name)
    .bind(input.template_text)
    .bind(input.version)
    .bind(input.model_type)
    .bind(input.is_active)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

pub async fn get_template(pool: &PgPool, id: Uuid) -> AppResult<PromptTemplate> {
    sqlx::query_as(
        r#"SELECT id, name, template_text, version, model_type, is_active, created_at
           FROM prompt_templates
           WHERE id = $1 AND deleted_at IS NULL"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("prompt template {id}")))
}

pub async fn list_templates(
    pool: &PgPool,
    params: &PageParams,
) -> AppResult<Page<PromptTemplate>> {
    let items = sqlx::query_as(
        r#"SELECT id, name, template_text, version, model_type, is_active, created_at
           FROM prompt_templates
           WHERE deleted_at IS NULL
           ORDER BY created_at DESC, id DESC
           LIMIT $1 OFFSET $2"#,
    )
    .bind(params.limit())
    .bind(params.offset())
    .fetch_all(pool)
    .await?;
    let total =
        sqlx::query_scalar("SELECT COUNT(*) FROM prompt_templates WHERE deleted_at IS NULL")
            .fetch_one(pool)
            .await?;
    Ok(Page::new(items, params, Some(total)))
}

fn validate_required(field: &str, value: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::Unprocessable(format!("{field} cannot be empty")));
    }
    Ok(())
}
