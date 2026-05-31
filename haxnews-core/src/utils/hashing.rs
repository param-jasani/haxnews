use sha2::{Sha256, Digest};

/// Compute a SHA256 hash of the input string
pub fn compute_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash() {
        let hash1 = compute_hash("test");
        let hash2 = compute_hash("test");
        assert_eq!(hash1, hash2);
        
        let hash3 = compute_hash("different");
        assert_ne!(hash1, hash3);
    }
}
