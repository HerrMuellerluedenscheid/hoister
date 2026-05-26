use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Hash an agent token for storage and lookup.
///
/// We HMAC the token with a server-side pepper rather than plain SHA-256.
/// That way a DB dump alone is not enough to confirm whether a stolen
/// `hst_` value matches a stored row — an attacker also needs the pepper,
/// which lives in the controller's deployment env. The HMAC itself is
/// fast; per-request hashing stays well under a millisecond.
///
/// `pepper` is normally read from `HOISTER_CONTROLLER_TOKEN_PEPPER`. An
/// empty pepper is accepted (the resulting HMAC degenerates toward the
/// unsalted-SHA256 case and gets the same security as before); callers
/// should warn loudly when running without one.
pub fn hash_token(token: &str, pepper: &[u8]) -> String {
    let mut mac =
        HmacSha256::new_from_slice(pepper).expect("HMAC-SHA256 accepts a key of any length");
    mac.update(token.as_bytes());
    hex(&mac.finalize().into_bytes())
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

    const PEPPER: &[u8] = b"test-pepper";

    #[test]
    fn hash_is_deterministic_and_hex() {
        let h = hash_token("hst_example", PEPPER);
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(h, hash_token("hst_example", PEPPER));
    }

    #[test]
    fn different_tokens_produce_different_hashes() {
        assert_ne!(hash_token("hst_a", PEPPER), hash_token("hst_b", PEPPER));
    }

    #[test]
    fn different_peppers_produce_different_hashes_for_the_same_token() {
        let h1 = hash_token("hst_example", b"pepper-one");
        let h2 = hash_token("hst_example", b"pepper-two");
        assert_ne!(h1, h2, "the pepper must be domain-separating");
    }

    #[test]
    fn empty_pepper_is_accepted() {
        // Acceptable as a fallback for self-hosted dev mode — the controller
        // logs a warning at startup in this case.
        let h = hash_token("hst_example", b"");
        assert_eq!(h.len(), 64);
    }
}
