//! Hash primitive for the audit chain.

use sha2::{Digest, Sha256};

/// Genesis hash (all-zero SHA-256, hex-encoded). Used as the predecessor
/// of the first real audit row.
pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// `sha256(prev_hash || payload)` as a lower-case hex string.
///
/// `prev_hash` and `payload` are concatenated as raw bytes — no
/// separator, no length prefix. They must be deterministic strings on
/// both sides of the verify boundary; this is why `payload` is always
/// produced by [`super::canonical::canonical_json`].
pub fn compute_hash(prev_hash: &str, payload: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prev_hash.as_bytes());
    hasher.update(payload.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let h1 = compute_hash(GENESIS_HASH, r#"{"a":1}"#);
        let h2 = compute_hash(GENESIS_HASH, r#"{"a":1}"#);
        assert_eq!(h1, h2);
        assert_ne!(h1, GENESIS_HASH);
    }

    #[test]
    fn different_payload_different_hash() {
        let h1 = compute_hash(GENESIS_HASH, r#"{"a":1}"#);
        let h2 = compute_hash(GENESIS_HASH, r#"{"a":2}"#);
        assert_ne!(h1, h2);
    }

    #[test]
    fn different_prev_different_hash() {
        let h1 = compute_hash(GENESIS_HASH, r#"{"a":1}"#);
        let h2 = compute_hash(&h1, r#"{"a":1}"#);
        assert_ne!(h1, h2);
    }
}
