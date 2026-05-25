use sha2::{Digest, Sha256};

/// Hash an agent token for storage / lookup. We deliberately use a plain
/// SHA-256 (not a slow KDF): tokens are 128 random bits, so brute-forcing
/// the preimage is infeasible without the protection that bcrypt/argon2
/// give weak passwords. Cheap hashing keeps the auth middleware fast.
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let bytes = hasher.finalize();
    hex(&bytes)
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(nibble(byte >> 4));
        out.push(nibble(byte & 0xf));
    }
    out
}

fn nibble(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        10..=15 => (b'a' + n - 10) as char,
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_deterministic_and_hex() {
        let h = hash_token("hst_example");
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(h, hash_token("hst_example"));
    }

    #[test]
    fn different_tokens_produce_different_hashes() {
        assert_ne!(hash_token("hst_a"), hash_token("hst_b"));
    }
}
