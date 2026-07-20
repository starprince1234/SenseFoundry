pub mod hybrid;
pub mod opensearch;
pub mod routes;
pub mod vector_search;

pub use hybrid::{reciprocal_rank_fusion, FusedResult};
pub use opensearch::{OpenSearchClient, OpenSearchHit};
pub use vector_search::{hnsw_search, VectorSearchResult};

pub const SENTENCES_INDEX: &str = "usage-sentences";

#[cfg(test)]
mod tests;
