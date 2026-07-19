use std::collections::HashMap; use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc}; use serde::{Deserialize, Serialize}; use uuid::Uuid;
#[derive(Clone, Debug, Deserialize, Serialize)] pub struct ManifestEdition { pub edition_id: Uuid, pub content_hash: String, pub signature: String, pub sequence: u64 }
#[derive(Clone, Debug, Serialize)] pub struct SyncManifest { pub id: Uuid, pub generated_at: DateTime<Utc>, pub sync_token: u64, pub editions: Vec<ManifestEdition> }
#[derive(Clone, Debug, Serialize)] pub struct DeltaResponse { pub manifest_id: Uuid, pub previous_sync_token: u64, pub sync_token: u64, pub changed_editions: Vec<ManifestEdition> }
#[derive(Clone, Default)] pub struct SyncService { manifests: Arc<RwLock<HashMap<Uuid, SyncManifest>>> }
impl SyncService {
    pub fn create_manifest(&self, editions: Vec<ManifestEdition>) -> SyncManifest { let sync_token = editions.iter().map(|edition| edition.sequence).max().unwrap_or(0); let manifest = SyncManifest { id: Uuid::new_v4(), generated_at: Utc::now(), sync_token, editions }; self.manifests.write().unwrap_or_else(|e| e.into_inner()).insert(manifest.id, manifest.clone()); manifest }
    pub fn list(&self) -> Vec<SyncManifest> { self.manifests.read().unwrap_or_else(|e| e.into_inner()).values().cloned().collect() }
    pub fn delta(&self, id: Uuid, last_sync_token: u64) -> Option<DeltaResponse> { let manifests = self.manifests.read().unwrap_or_else(|e| e.into_inner()); let manifest = manifests.get(&id)?; Some(DeltaResponse { manifest_id: id, previous_sync_token: last_sync_token, sync_token: manifest.sync_token, changed_editions: manifest.editions.iter().filter(|edition| edition.sequence > last_sync_token).cloned().collect() }) }
}
#[cfg(test)] mod tests { use super::*; #[test] fn returns_only_editions_after_token() { let service = SyncService::default(); let manifest = service.create_manifest(vec![ManifestEdition { edition_id: Uuid::new_v4(), content_hash: "a".into(), signature: "s".into(), sequence: 1 }, ManifestEdition { edition_id: Uuid::new_v4(), content_hash: "b".into(), signature: "t".into(), sequence: 2 }]); let delta = service.delta(manifest.id, 1).expect("delta"); assert_eq!(delta.changed_editions.len(), 1); assert_eq!(delta.changed_editions[0].sequence, 2); } }
