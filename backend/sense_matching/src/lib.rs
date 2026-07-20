pub mod matcher;
pub mod routes;
pub mod unknown_pool;

pub use matcher::{detect_unknown, MatchResult, SenseMatcher, RERANK_THRESHOLD};
pub use routes::routes;
pub use unknown_pool::{add_to_unknown_pool, list_unknown_pool, UnknownPoolEntry};

#[cfg(test)]
mod tests;
