use kernel::{AppError, AppResult};
use sqlx::PgPool;

use crate::{NewReferenceSense, SourceKind};

pub struct IsolationRules;

impl IsolationRules {
    /// Internal bootstrap content can aid matching but can never become publication content.
    pub fn validate_seed_sense(sense: &NewReferenceSense) -> AppResult<()> {
        if sense.source_kind == SourceKind::InternalSeed {
            if sense.is_authoritative {
                return Err(AppError::Unprocessable(
                    "Internal seed senses cannot be marked authoritative".into(),
                ));
            }
            if sense.is_publishable {
                return Err(AppError::Unprocessable(
                    "Internal seed senses cannot be marked publishable".into(),
                ));
            }
        }
        Ok(())
    }

    pub fn check_authoritative_gate(is_authoritative: bool) -> AppResult<()> {
        if is_authoritative {
            let enabled = std::env::var("AUTHORITATIVE_DICT_ENABLED")
                .unwrap_or_default()
                .to_lowercase();
            if enabled != "true" {
                return Err(AppError::Forbidden(
                    "AUTHORITATIVE_DICT_ENABLED gate is not open. Set env var to import authoritative content."
                        .into(),
                ));
            }
        }
        Ok(())
    }

    pub(crate) async fn validate_source_record(
        pool: &PgPool,
        sense: &NewReferenceSense,
    ) -> AppResult<Option<uuid::Uuid>> {
        let source = sqlx::query_as::<_, (String, bool)>(
            "SELECT source_kind, is_authoritative FROM sources WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(sense.source_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Unprocessable("Reference source does not exist".into()))?;

        let source_matches = match sense.source_kind {
            SourceKind::InternalSeed => source.0 == "internal_seed" && !source.1,
            SourceKind::UserSubmitted => source.0 == "user_submission" && !source.1,
            SourceKind::Authoritative => source.0 != "internal_seed" && source.1,
        };
        if !source_matches {
            return Err(AppError::Unprocessable(
                "Sense kind and authority must match its isolated source record".into(),
            ));
        }

        let existing_kind = sqlx::query_scalar::<_, String>(
            "SELECT source_kind FROM reference_senses WHERE source_id = $1 AND deleted_at IS NULL LIMIT 1",
        )
        .bind(sense.source_id)
        .fetch_optional(pool)
        .await?;
        if existing_kind.as_deref().is_some_and(|kind| kind != sense.source_kind.as_db_str()) {
            return Err(AppError::Conflict(
                "Seed and authoritative content cannot share a source record".into(),
            ));
        }

        if sense.source_kind != SourceKind::Authoritative {
            return Ok(None);
        }

        let versions = sqlx::query_scalar::<_, uuid::Uuid>(
            "SELECT v.id FROM reference_dictionary_versions v \
             JOIN reference_dictionaries d ON d.id = v.dictionary_id \
             WHERE d.source_id = $1 AND d.deleted_at IS NULL AND v.deleted_at IS NULL \
             ORDER BY v.created_at DESC LIMIT 2",
        )
        .bind(sense.source_id)
        .fetch_all(pool)
        .await?;

        match versions.as_slice() {
            [version_id] => Ok(Some(*version_id)),
            [] => Err(AppError::Unprocessable(
                "Authoritative senses require an isolated dictionary version".into(),
            )),
            _ => Err(AppError::Unprocessable(
                "Source has multiple dictionary versions; import through a version-specific pipeline".into(),
            )),
        }
    }
}
