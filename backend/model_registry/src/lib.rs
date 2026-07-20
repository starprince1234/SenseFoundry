use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

pub mod routes;
pub mod service;
pub mod version_binding;

pub use routes::routes;
pub use service::{
    create_model, create_template, get_model, get_template, list_models, list_templates,
    update_model, CreateModel, CreatePromptTemplate, UpdateModel,
};
pub use version_binding::{apply_to_usage_instance, resolve_active_binding};

pub const DEFAULT_MODELS: &[(&str, &str, &str)] = &[
    ("bert-base-chinese", "main", "mlm"),
    ("BAAI/bge-reranker-base", "main", "reranker"),
];
pub const INITIAL_FEATURE_VERSION: &str = "0.1.0";
pub const INITIAL_DATA_VERSION: &str = "0.1.0";

/// A registered model version.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ModelVersion {
    pub id: Uuid,
    pub name: String,
    pub revision: String,
    pub model_type: ModelType,
    pub feature_version: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum ModelType {
    Mlm,
    Reranker,
    Embedding,
    Llm,
}

/// A prompt template for LLM definition drafting.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PromptTemplate {
    pub id: Uuid,
    pub name: String,
    pub template_text: String,
    pub version: String,
    pub model_type: ModelType,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Exact versions needed to reproduce an inference output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionBinding {
    pub model_version: String,
    pub feature_version: String,
    pub prompt_template_id: Option<Uuid>,
    pub data_version: String,
}

#[cfg(test)]
mod tests;
