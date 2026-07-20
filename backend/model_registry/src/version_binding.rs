use kernel::{AppError, AppResult};
use sqlx::PgPool;

use crate::{ModelType, VersionBinding, INITIAL_DATA_VERSION};

/// Resolves the active versions that every inference output must carry.
pub async fn resolve_active_binding(
    pool: &PgPool,
    model_type: ModelType,
) -> Result<VersionBinding, AppError> {
    let model = sqlx::query_as::<_, ActiveModel>(
        r#"SELECT model_version, feature_version
           FROM model_registry
           WHERE model_type = $1 AND is_active = TRUE AND deleted_at IS NULL
           ORDER BY created_at DESC, id DESC
           LIMIT 1"#,
    )
    .bind(model_type)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("active {model_type:?} model")))?;

    let prompt_template_id = sqlx::query_scalar(
        r#"SELECT id
           FROM prompt_templates
           WHERE model_type = $1 AND is_active = TRUE AND deleted_at IS NULL
           ORDER BY created_at DESC, id DESC
           LIMIT 1"#,
    )
    .bind(model_type)
    .fetch_optional(pool)
    .await?;

    build_binding(model.model_version, model.feature_version, prompt_template_id)
}

/// Extracts the mandatory model and feature versions for a usage instance.
pub fn apply_to_usage_instance(binding: &VersionBinding) -> (String, String) {
    (
        binding.model_version.clone(),
        binding.feature_version.clone(),
    )
}

#[derive(sqlx::FromRow)]
struct ActiveModel {
    model_version: String,
    feature_version: String,
}

fn build_binding(
    model_version: String,
    feature_version: String,
    prompt_template_id: Option<uuid::Uuid>,
) -> AppResult<VersionBinding> {
    if model_version.is_empty() || feature_version.is_empty() {
        return Err(AppError::Internal(anyhow::anyhow!(
            "active model has incomplete version metadata"
        )));
    }
    Ok(VersionBinding {
        model_version,
        feature_version,
        prompt_template_id,
        data_version: INITIAL_DATA_VERSION.to_owned(),
    })
}
