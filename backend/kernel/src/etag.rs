use hex::encode;
use sha2::{Digest, Sha256};

pub fn compute_etag(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("\"{}\"", encode(hasher.finalize()))
}

pub fn check_if_match(etag: &str, if_match_header: Option<&str>) -> bool {
    match if_match_header {
        None => true,
        Some(header) => header == etag || header == "*",
    }
}
