pub mod copyright;
pub mod routes;
pub mod service;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

pub use copyright::CopyrightStatus;
pub use routes::routes;
pub use service::{validate_doi, validate_isbn};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Source {
    pub id: Uuid,
    pub uri: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub doi: Option<String>,
    pub license: Option<String>,
    pub copyright_status: CopyrightStatus,
    pub is_storable: bool,
    pub is_trainable: bool,
    pub is_publishable: bool,
    pub source_kind: SourceKind,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum SourceKind {
    WebPage,
    Book,
    Journal,
    UserSubmission,
    InternalSeed,
}

impl SourceKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::WebPage => "web_page",
            Self::Book => "book",
            Self::Journal => "journal",
            Self::UserSubmission => "user_submission",
            Self::InternalSeed => "internal_seed",
        }
    }
}

#[cfg(test)]
mod tests;
