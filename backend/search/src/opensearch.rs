use anyhow::Context;
use kernel::AppError;
use reqwest::{Client, StatusCode};
use uuid::Uuid;

#[derive(Clone)]
pub struct OpenSearchClient {
    client: Client,
    base_url: String,
}

impl OpenSearchClient {
    pub fn new(url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: url.trim_end_matches('/').to_string(),
        }
    }

    /// Full-text search for sentences containing the target headword.
    pub async fn search_sentences(
        &self,
        index: &str,
        target_headword: &str,
        query_text: Option<&str>,
        size: usize,
    ) -> Result<Vec<OpenSearchHit>, AppError> {
        let query = if let Some(query_text) = query_text {
            serde_json::json!({
                "query": {
                    "bool": {
                        "must": [
                            {"term": {"target_headword": target_headword}},
                            {"match": {"sentence_text": query_text}}
                        ]
                    }
                },
                "size": size
            })
        } else {
            serde_json::json!({
                "query": {"term": {"target_headword": target_headword}},
                "size": size
            })
        };

        let response = self
            .client
            .post(format!("{}/{index}/_search", self.base_url))
            .json(&query)
            .send()
            .await
            .context("OpenSearch request failed")?
            .error_for_status()
            .context("OpenSearch search returned an error")?;
        let body: serde_json::Value = response
            .json()
            .await
            .context("invalid OpenSearch search response")?;

        Ok(body["hits"]["hits"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|hit| {
                let usage_instance_id = hit["_source"]["usage_instance_id"]
                    .as_str()?
                    .parse::<Uuid>()
                    .ok()?;
                Some(OpenSearchHit {
                    usage_instance_id,
                    bm25_score: hit["_score"].as_f64().unwrap_or(0.0),
                })
            })
            .collect())
    }

    /// Create the sentence index with the configured Chinese analyzer.
    pub async fn create_index(&self, index: &str) -> Result<(), AppError> {
        let mapping = serde_json::json!({
            "mappings": {
                "properties": {
                    "usage_instance_id": {"type": "keyword"},
                    "target_headword": {"type": "keyword"},
                    "sentence_text": {"type": "text", "analyzer": "ik_smart"},
                    "context_window": {"type": "text", "analyzer": "ik_smart"}
                }
            }
        });
        let response = self
            .client
            .put(format!("{}/{index}", self.base_url))
            .json(&mapping)
            .send()
            .await
            .context("OpenSearch index creation request failed")?;

        if response.status().is_success() || response.status() == StatusCode::BAD_REQUEST {
            return Ok(());
        }
        Err(AppError::Internal(anyhow::anyhow!(
            "OpenSearch index creation failed with status {}",
            response.status()
        )))
    }

    pub async fn health_check(&self) -> Result<bool, AppError> {
        let response = self
            .client
            .get(format!("{}/_cluster/health", self.base_url))
            .send()
            .await
            .context("OpenSearch health check failed")?;
        Ok(response.status().is_success())
    }
}

#[derive(Debug, Clone)]
pub struct OpenSearchHit {
    pub usage_instance_id: Uuid,
    pub bm25_score: f64,
}
