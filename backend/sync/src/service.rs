use std::sync::Arc;

use chrono::{DateTime, Utc};
use publication::{Edition, PublicationError, PublicationService};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ManifestEdition {
    pub edition_id: Uuid,
    pub content_hash: String,
    pub signature: String,
    pub sequence: u64,
}

impl From<Edition> for ManifestEdition {
    fn from(edition: Edition) -> Self {
        Self {
            edition_id: edition.id,
            content_hash: edition.content_hash,
            signature: edition.signature,
            sequence: edition.sequence,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SyncManifest {
    pub id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub sync_token: u64,
    pub editions: Vec<ManifestEdition>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DeltaResponse {
    pub manifest_id: Uuid,
    pub previous_sync_token: u64,
    pub sync_token: u64,
    pub changed_editions: Vec<ManifestEdition>,
}

#[derive(Clone)]
pub struct SyncService {
    publication_service: Arc<PublicationService>,
    manifest_id: Uuid,
    generated_at: DateTime<Utc>,
}

impl SyncService {
    pub fn new(publication_service: Arc<PublicationService>) -> Self {
        Self {
            publication_service,
            manifest_id: Uuid::new_v4(),
            generated_at: Utc::now(),
        }
    }

    pub fn latest_manifest(&self) -> SyncManifest {
        let editions: Vec<_> = self
            .publication_service
            .list()
            .into_iter()
            .map(ManifestEdition::from)
            .collect();
        let sync_token = editions
            .iter()
            .map(|edition| edition.sequence)
            .max()
            .unwrap_or(0);
        SyncManifest {
            id: self.manifest_id,
            generated_at: self.generated_at,
            sync_token,
            editions,
        }
    }

    pub fn list(&self) -> Vec<SyncManifest> {
        vec![self.latest_manifest()]
    }

    pub fn latest_delta(&self, last_sync_token: u64) -> DeltaResponse {
        let manifest = self.latest_manifest();
        DeltaResponse {
            manifest_id: manifest.id,
            previous_sync_token: last_sync_token,
            sync_token: manifest.sync_token,
            changed_editions: manifest
                .editions
                .into_iter()
                .filter(|edition| edition.sequence > last_sync_token)
                .collect(),
        }
    }

    pub fn delta(
        &self,
        manifest_id: Uuid,
        last_sync_token: u64,
    ) -> Option<DeltaResponse> {
        (manifest_id == self.manifest_id).then(|| self.latest_delta(last_sync_token))
    }

    pub fn edition_delta(&self, edition_id: Uuid) -> Result<Vec<u8>, PublicationError> {
        self.publication_service.delta(edition_id)
    }
}
