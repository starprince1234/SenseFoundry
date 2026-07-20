pub mod isolation;
pub mod routes;
pub mod service;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use isolation::IsolationRules;
pub use routes::routes;
pub use service::{import_seed_senses, DictionaryImportService};

pub const MVP_HEADWORDS: &[&str] = &["打", "开", "发", "上", "下"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewReferenceSense {
    pub headword_id: Uuid,
    pub source_id: Uuid,
    pub sense_number: i32,
    pub pos: Option<String>,
    pub gloss: String,
    pub example_text: Option<String>,
    pub source_kind: SourceKind,
    pub is_authoritative: bool,
    pub is_publishable: bool,
    pub copyright_isolate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Authoritative,
    InternalSeed,
    UserSubmitted,
}

impl SourceKind {
    pub(crate) const fn as_db_str(self) -> &'static str {
        match self {
            Self::Authoritative => "authoritative",
            Self::InternalSeed => "internal_seed",
            Self::UserSubmitted => "user_submitted",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewHeadword {
    pub character: String,
    pub pinyin: Vec<String>,
    pub stroke_count: Option<i32>,
    pub radical: Option<String>,
    pub traditional_form: Option<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Headword {
    pub id: Uuid,
    pub character: String,
    pub normalized: Option<String>,
    pub pinyin: Option<Vec<String>>,
    pub stroke_count: Option<i32>,
    pub radical: Option<String>,
    pub traditional_form: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ReferenceSense {
    pub id: Uuid,
    pub headword_id: Uuid,
    pub source_id: Uuid,
    pub dictionary_version_id: Option<Uuid>,
    pub sense_number: Option<i32>,
    pub pos: Option<String>,
    pub gloss: String,
    pub example_text: Option<String>,
    pub source_kind: String,
    pub is_authoritative: bool,
    pub is_publishable: bool,
    pub copyright_isolate: bool,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests;
