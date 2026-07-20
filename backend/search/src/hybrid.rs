use std::collections::HashMap;

use serde::Serialize;
use uuid::Uuid;

use crate::{OpenSearchHit, VectorSearchResult};

/// Reciprocal Rank Fusion of BM25 and cosine-similarity result rankings.
pub fn reciprocal_rank_fusion(
    bm25_hits: &[OpenSearchHit],
    vector_hits: &[VectorSearchResult],
    k: f64,
) -> Vec<FusedResult> {
    let mut scores: HashMap<Uuid, f64> = HashMap::new();

    for (rank, hit) in bm25_hits.iter().enumerate() {
        *scores.entry(hit.usage_instance_id).or_insert(0.0) +=
            1.0 / (k + rank as f64 + 1.0);
    }
    for (rank, hit) in vector_hits.iter().enumerate() {
        *scores.entry(hit.id).or_insert(0.0) += 1.0 / (k + rank as f64 + 1.0);
    }

    let mut results: Vec<_> = scores
        .into_iter()
        .map(|(usage_instance_id, fused_score)| FusedResult {
            usage_instance_id,
            fused_score,
        })
        .collect();
    results.sort_by(|left, right| {
        right
            .fused_score
            .total_cmp(&left.fused_score)
            .then_with(|| left.usage_instance_id.cmp(&right.usage_instance_id))
    });
    results
}

#[derive(Debug, Serialize)]
pub struct FusedResult {
    pub usage_instance_id: Uuid,
    pub fused_score: f64,
}
