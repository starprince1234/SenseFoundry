use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Edition { pub id: Uuid, pub headword: String, pub version_number: u32, pub content: Value, pub diff_snapshot: Value, pub content_hash: String, pub signature: String, pub publisher_id: Uuid, pub rollback_of: Option<Uuid>, pub created_at: DateTime<Utc> }
#[derive(Debug, thiserror::Error)] pub enum PublicationError { #[error("edition not found")] NotFound, #[error("content serialization failed: {0}")] Serialization(String) }
#[derive(Clone, Default)] pub struct PublicationService { editions: Arc<RwLock<Vec<Edition>>> }

impl PublicationService {
    pub fn publish(&self, headword: &str, content: Value, publisher_id: Uuid) -> Result<Edition, PublicationError> { self.create_edition(headword, content, publisher_id, None) }
    fn create_edition(&self, headword: &str, content: Value, publisher_id: Uuid, rollback_of: Option<Uuid>) -> Result<Edition, PublicationError> {
        let mut editions = self.editions.write().unwrap_or_else(|e| e.into_inner());
        let previous = editions.iter().filter(|edition| edition.headword == headword).max_by_key(|edition| edition.version_number);
        let version_number = previous.map_or(1, |edition| edition.version_number + 1);
        let id = Uuid::new_v4();
        let bytes = serde_json::to_vec(&content).map_err(|error| PublicationError::Serialization(error.to_string()))?;
        let content_hash = hex::encode(Sha256::digest(&bytes));
        let signature = hex::encode(Sha256::digest(format!("{content_hash}:{id}:{publisher_id}").as_bytes()));
        let diff_snapshot = json_diff(previous.map(|edition| &edition.content), &content);
        let edition = Edition { id, headword: headword.into(), version_number, content, diff_snapshot, content_hash, signature, publisher_id, rollback_of, created_at: Utc::now() };
        editions.push(edition.clone()); Ok(edition)
    }
    pub fn list(&self) -> Vec<Edition> { self.editions.read().unwrap_or_else(|e| e.into_inner()).clone() }
    pub fn rollback(&self, id: Uuid, publisher_id: Uuid) -> Result<Edition, PublicationError> { let target = self.editions.read().unwrap_or_else(|e| e.into_inner()).iter().find(|edition| edition.id == id).cloned().ok_or(PublicationError::NotFound)?; self.create_edition(&target.headword, target.content, publisher_id, Some(id)) }
}
fn json_diff(old: Option<&Value>, new: &Value) -> Value {
    let old_object = old.and_then(Value::as_object); let new_object = new.as_object(); let mut changed = HashMap::new();
    if let Some(new_object) = new_object { for (key, value) in new_object { let before = old_object.and_then(|object| object.get(key)); if before != Some(value) { changed.insert(key.clone(), serde_json::json!({"old": before, "new": value})); } } }
    serde_json::json!({"changed": changed})
}
#[cfg(test)] mod tests { use super::*; #[test] fn versions_signs_and_rolls_back() { let service = PublicationService::default(); let publisher = Uuid::new_v4(); let first = service.publish("bank", serde_json::json!({"sense":"edge"}), publisher).expect("publish"); let second = service.publish("bank", serde_json::json!({"sense":"financial"}), publisher).expect("publish"); assert_eq!(second.version_number, 2); assert!(!second.signature.is_empty()); let rollback = service.rollback(first.id, publisher).expect("rollback"); assert_eq!(rollback.version_number, 3); assert_eq!(rollback.rollback_of, Some(first.id)); } }
