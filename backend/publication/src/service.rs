use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{DateTime, Utc};
use p256::ecdsa::{
    signature::Signer,
    Signature, SigningKey,
};
use p256::pkcs8::DecodePrivateKey;
use review::{ReviewService, ReviewState};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Edition {
    pub id: Uuid,
    pub sequence: u64,
    pub headword: String,
    pub version_number: u32,
    pub content: Value,
    pub diff_snapshot: Value,
    pub content_hash: String,
    pub signature: String,
    pub review_task_id: Uuid,
    pub publisher_id: Uuid,
    pub rollback_of: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    #[serde(skip)]
    delta_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PublicationPreview {
    pub headword: String,
    pub version_number: u32,
    pub diff_snapshot: Value,
    pub content_hash: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PublicationError {
    #[error("edition not found")]
    NotFound,
    #[error("edition content must contain senses and examples arrays")]
    InvalidContent,
    #[error("review task not found")]
    ReviewNotFound,
    #[error("publication requires a completed review task with two approvals")]
    ReviewRequired,
    #[error("review task was approved for a different delta content hash")]
    ReviewedContentMismatch,
    #[error("SYNC_SIGNING_PRIVATE_KEY is not a valid P-256 PKCS#8 PEM key: {0}")]
    InvalidSigningKey(String),
    #[error("content serialization failed: {0}")]
    Serialization(String),
}

#[derive(Serialize)]
struct DeltaPackage<'a> {
    headword: &'a str,
    version_number: u32,
    senses: &'a Value,
    examples: &'a Value,
}

#[derive(Clone)]
pub struct PublicationService {
    editions: Arc<RwLock<Vec<Edition>>>,
    review_service: Arc<ReviewService>,
    signing_key: Arc<SigningKey>,
}

impl PublicationService {
    pub fn new(
        review_service: Arc<ReviewService>,
        signing_private_key_pem: &str,
    ) -> Result<Self, PublicationError> {
        let signing_key = SigningKey::from_pkcs8_pem(signing_private_key_pem)
            .map_err(|error| PublicationError::InvalidSigningKey(error.to_string()))?;
        Ok(Self {
            editions: Arc::new(RwLock::new(Vec::new())),
            review_service,
            signing_key: Arc::new(signing_key),
        })
    }

    pub fn preview(
        &self,
        headword: &str,
        content: &Value,
    ) -> Result<PublicationPreview, PublicationError> {
        let editions = self
            .editions
            .read()
            .unwrap_or_else(|error| error.into_inner());
        let previous = latest_for_headword(&editions, headword);
        let version_number = previous.map_or(1, |edition| edition.version_number + 1);
        let delta_bytes = delta_bytes(headword, version_number, content)?;
        Ok(PublicationPreview {
            headword: headword.to_owned(),
            version_number,
            diff_snapshot: json_diff(previous.map(|edition| &edition.content), content),
            content_hash: content_hash(&delta_bytes),
        })
    }

    pub fn publish(
        &self,
        headword: &str,
        content: Value,
        publisher_id: Uuid,
        review_task_id: Uuid,
    ) -> Result<Edition, PublicationError> {
        self.create_edition(
            headword,
            content,
            publisher_id,
            review_task_id,
            None,
        )
    }

    fn create_edition(
        &self,
        headword: &str,
        content: Value,
        publisher_id: Uuid,
        review_task_id: Uuid,
        rollback_of: Option<Uuid>,
    ) -> Result<Edition, PublicationError> {
        let mut editions = self
            .editions
            .write()
            .unwrap_or_else(|error| error.into_inner());
        let previous = latest_for_headword(&editions, headword);
        let version_number = previous.map_or(1, |edition| edition.version_number + 1);
        let delta_bytes = delta_bytes(headword, version_number, &content)?;
        let content_hash = content_hash(&delta_bytes);
        self.verify_review_gate(review_task_id, &content_hash)?;

        let signature: Signature = self.signing_key.sign(content_hash.as_bytes());
        let edition = Edition {
            id: Uuid::new_v4(),
            sequence: editions.len() as u64 + 1,
            headword: headword.to_owned(),
            version_number,
            diff_snapshot: json_diff(previous.map(|edition| &edition.content), &content),
            content,
            content_hash,
            signature: BASE64.encode(signature.to_der().as_bytes()),
            review_task_id,
            publisher_id,
            rollback_of,
            created_at: Utc::now(),
            delta_bytes,
        };
        editions.push(edition.clone());
        Ok(edition)
    }

    fn verify_review_gate(
        &self,
        review_task_id: Uuid,
        content_hash: &str,
    ) -> Result<(), PublicationError> {
        let task = self
            .review_service
            .get(review_task_id)
            .map_err(|_| PublicationError::ReviewNotFound)?;
        if task.state != ReviewState::Completed || !task.publishable {
            return Err(PublicationError::ReviewRequired);
        }
        if task.reviewed_content_hash != content_hash {
            return Err(PublicationError::ReviewedContentMismatch);
        }
        Ok(())
    }

    pub fn list(&self) -> Vec<Edition> {
        self.editions
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    pub fn get(&self, id: Uuid) -> Result<Edition, PublicationError> {
        self.editions
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .iter()
            .find(|edition| edition.id == id)
            .cloned()
            .ok_or(PublicationError::NotFound)
    }

    pub fn delta(&self, id: Uuid) -> Result<Vec<u8>, PublicationError> {
        self.get(id).map(|edition| edition.delta_bytes)
    }

    pub fn rollback(
        &self,
        id: Uuid,
        publisher_id: Uuid,
        review_task_id: Uuid,
    ) -> Result<Edition, PublicationError> {
        let target = self.get(id)?;
        self.create_edition(
            &target.headword,
            target.content,
            publisher_id,
            review_task_id,
            Some(id),
        )
    }
}

fn latest_for_headword<'a>(editions: &'a [Edition], headword: &str) -> Option<&'a Edition> {
    editions
        .iter()
        .filter(|edition| edition.headword == headword)
        .max_by_key(|edition| edition.version_number)
}

fn delta_bytes(
    headword: &str,
    version_number: u32,
    content: &Value,
) -> Result<Vec<u8>, PublicationError> {
    let senses = content
        .get("senses")
        .filter(|value| value.is_array())
        .ok_or(PublicationError::InvalidContent)?;
    let examples = content
        .get("examples")
        .filter(|value| value.is_array())
        .ok_or(PublicationError::InvalidContent)?;
    serde_json::to_vec(&DeltaPackage {
        headword,
        version_number,
        senses,
        examples,
    })
    .map_err(|error| PublicationError::Serialization(error.to_string()))
}

fn content_hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn json_diff(old: Option<&Value>, new: &Value) -> Value {
    let old_object = old.and_then(Value::as_object);
    let new_object = new.as_object();
    let mut changed = HashMap::new();
    if let Some(new_object) = new_object {
        for (key, value) in new_object {
            let before = old_object.and_then(|object| object.get(key));
            if before != Some(value) {
                changed.insert(
                    key.clone(),
                    serde_json::json!({"old": before, "new": value}),
                );
            }
        }
    }
    serde_json::json!({"changed": changed})
}

#[cfg(test)]
mod tests {
    use p256::ecdsa::{signature::Verifier, VerifyingKey};
    use p256::pkcs8::{EncodePrivateKey, LineEnding};
    use rand_core::OsRng;
    use review::Decision;

    use super::*;

    fn service() -> (PublicationService, Arc<ReviewService>) {
        let review_service = Arc::new(ReviewService::default());
        let signing_key = SigningKey::random(&mut OsRng);
        let private_key = signing_key
            .to_pkcs8_pem(LineEnding::LF)
            .expect("encode test key");
        (
            PublicationService::new(review_service.clone(), private_key.as_str())
                .expect("create publication service"),
            review_service,
        )
    }

    fn content() -> Value {
        serde_json::json!({
            "senses": [{"id": "sense-1", "pos": "verb", "definition": "敲击"}],
            "examples": [{"id": "example-1", "sense_id": "sense-1", "sentence": "打鼓。", "rank": 1.0}]
        })
    }

    fn approve(
        service: &PublicationService,
        reviews: &ReviewService,
        headword: &str,
        content: &Value,
    ) -> Uuid {
        let first = Uuid::new_v4();
        let second = Uuid::new_v4();
        let preview = service.preview(headword, content).expect("preview");
        let task = reviews
            .create(
                Uuid::new_v4(),
                vec![first, second],
                preview.content_hash,
            )
            .expect("create review");
        reviews
            .decide(task.id, first, Decision::Approve, None)
            .expect("first approval");
        reviews
            .decide(task.id, second, Decision::Approve, None)
            .expect("second approval");
        task.id
    }

    #[test]
    fn signs_exact_delta_hash_and_supports_reviewed_rollback() {
        let (service, reviews) = service();
        let publisher = Uuid::new_v4();
        let first_content = content();
        let review_task_id = approve(&service, &reviews, "打", &first_content);
        let first = service
            .publish("打", first_content, publisher, review_task_id)
            .expect("publish");
        assert_eq!(content_hash(&service.delta(first.id).expect("delta")), first.content_hash);

        let signature_bytes = BASE64.decode(&first.signature).expect("base64 signature");
        let signature = Signature::from_der(&signature_bytes).expect("DER signature");
        let verifying_key = VerifyingKey::from(&*service.signing_key);
        verifying_key
            .verify(first.content_hash.as_bytes(), &signature)
            .expect("verify signature");

        let rollback_review = approve(&service, &reviews, "打", &first.content);
        let rollback = service
            .rollback(first.id, publisher, rollback_review)
            .expect("rollback");
        assert_eq!(rollback.version_number, 2);
        assert_eq!(rollback.rollback_of, Some(first.id));
    }

    #[test]
    fn blocks_publication_when_reviewed_hash_does_not_match() {
        let (service, reviews) = service();
        let reviewed = content();
        let review_task_id = approve(&service, &reviews, "打", &reviewed);
        let changed = serde_json::json!({"senses": [], "examples": []});
        let error = service
            .publish("打", changed, Uuid::new_v4(), review_task_id)
            .expect_err("changed content must be rejected");
        assert!(matches!(error, PublicationError::ReviewedContentMismatch));
    }
}
