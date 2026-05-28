//! Envelope AEAD for notifier configs (and any future secret-bearing
//! column). One controller-wide key, AES-256-GCM, random 96-bit nonce per
//! record. Output format on disk is the JSON object
//!
//! ```json
//! { "v": 1, "n": "<base64 nonce>", "c": "<base64 ciphertext+tag>" }
//! ```
//!
//! kept inside the same TEXT/JSONB column. Pre-existing plaintext
//! `NotifierConfig` JSON is still readable: [`Aead::decrypt_or_plaintext`]
//! falls through to a normal JSON parse when the wrapper shape is absent,
//! and the next write will store the row encrypted. That gives us an
//! online migration without a separate backfill pass.
//!
//! Production key supplied via `HOISTER_CONTROLLER_NOTIFIER_KEY` (base64
//! of 32 bytes). If the env var is missing, the controller still starts
//! — useful for `cargo run` / self-hosted SQLite dev — but loudly logs a
//! warning and stores plaintext. The same flag flips reads back to
//! plaintext mode; encrypted rows then become inaccessible until the key
//! is restored. Operators should treat the key like the existing
//! `HOISTER_CONTROLLER_TOKEN_PEPPER`: rotate carefully, back up offline.

use aws_lc_rs::aead::{AES_256_GCM, Aad, LessSafeKey, NONCE_LEN, Nonce, UnboundKey};
use aws_lc_rs::rand::{SecureRandom, SystemRandom};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const WRAPPER_VERSION: u8 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct Wrapper {
    v: u8,
    n: String,
    c: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AeadError {
    #[error("invalid key (must be base64 of 32 bytes)")]
    InvalidKey,
    #[error("ciphertext too short")]
    Truncated,
    #[error("decryption failed")]
    DecryptFailed,
    #[error("encryption failed")]
    EncryptFailed,
    #[error("nonce generation failed")]
    NonceFailed,
}

#[derive(Clone)]
pub struct Aead {
    /// `None` = no key configured; encrypt/decrypt becomes a passthrough
    /// returning the input unchanged. Loud warning at construction time.
    inner: Option<Arc<LessSafeKey>>,
}

impl Aead {
    /// Build from base64-encoded 32-byte key. Empty / missing input
    /// returns a no-op Aead (passthrough); the caller is expected to
    /// have warned the operator.
    pub fn from_base64_or_passthrough(b64: Option<&str>) -> Result<Self, AeadError> {
        let Some(raw) = b64.filter(|s| !s.is_empty()) else {
            return Ok(Self { inner: None });
        };
        let bytes = BASE64_STANDARD
            .decode(raw.trim())
            .map_err(|_| AeadError::InvalidKey)?;
        if bytes.len() != 32 {
            return Err(AeadError::InvalidKey);
        }
        let unbound = UnboundKey::new(&AES_256_GCM, &bytes).map_err(|_| AeadError::InvalidKey)?;
        Ok(Self {
            inner: Some(Arc::new(LessSafeKey::new(unbound))),
        })
    }

    pub fn is_active(&self) -> bool {
        self.inner.is_some()
    }

    /// Encrypt `plaintext` and return a JSON wrapper string ready for
    /// storage. When the key is absent, returns the plaintext unchanged
    /// — caller stores it as-is, decrypt_or_plaintext will read it back.
    pub fn encrypt(&self, plaintext: &str) -> Result<String, AeadError> {
        let Some(key) = self.inner.as_ref() else {
            return Ok(plaintext.to_string());
        };
        let mut nonce_bytes = [0u8; NONCE_LEN];
        SystemRandom::new()
            .fill(&mut nonce_bytes)
            .map_err(|_| AeadError::NonceFailed)?;
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let mut in_out = plaintext.as_bytes().to_vec();
        key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
            .map_err(|_| AeadError::EncryptFailed)?;
        let wrapper = Wrapper {
            v: WRAPPER_VERSION,
            n: BASE64_STANDARD.encode(nonce_bytes),
            c: BASE64_STANDARD.encode(&in_out),
        };
        serde_json::to_string(&wrapper).map_err(|_| AeadError::EncryptFailed)
    }

    /// Read a stored value. If the wrapper shape is present, decrypt;
    /// otherwise return the input as-is so callers can still parse
    /// pre-encryption plaintext rows. The next write through `encrypt`
    /// will upgrade the row.
    pub fn decrypt_or_plaintext(&self, stored: &str) -> Result<String, AeadError> {
        // Lazy match: look for the `"v":` discriminator. Avoids parsing
        // legitimate notifier-config JSON (which has `"kind":...`) twice.
        let looks_wrapped =
            stored.contains("\"v\"") && stored.contains("\"n\"") && stored.contains("\"c\"");
        if !looks_wrapped {
            return Ok(stored.to_string());
        }
        let wrapper: Wrapper = match serde_json::from_str(stored) {
            Ok(w) => w,
            Err(_) => return Ok(stored.to_string()),
        };
        let Some(key) = self.inner.as_ref() else {
            // We have a wrapped row but no key — refuse rather than
            // returning meaningless bytes. Surfaces a config error
            // loudly.
            return Err(AeadError::DecryptFailed);
        };
        if wrapper.v != WRAPPER_VERSION {
            return Err(AeadError::DecryptFailed);
        }
        let nonce_bytes = BASE64_STANDARD
            .decode(wrapper.n.as_bytes())
            .map_err(|_| AeadError::Truncated)?;
        if nonce_bytes.len() != NONCE_LEN {
            return Err(AeadError::Truncated);
        }
        let mut nonce_arr = [0u8; NONCE_LEN];
        nonce_arr.copy_from_slice(&nonce_bytes);
        let nonce = Nonce::assume_unique_for_key(nonce_arr);
        let mut buf = BASE64_STANDARD
            .decode(wrapper.c.as_bytes())
            .map_err(|_| AeadError::Truncated)?;
        let pt = key
            .open_in_place(nonce, Aad::empty(), &mut buf)
            .map_err(|_| AeadError::DecryptFailed)?;
        String::from_utf8(pt.to_vec()).map_err(|_| AeadError::DecryptFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dev_key() -> String {
        BASE64_STANDARD.encode([7u8; 32])
    }

    #[test]
    fn round_trip_with_key() {
        let aead = Aead::from_base64_or_passthrough(Some(&dev_key())).unwrap();
        assert!(aead.is_active());
        let pt = r##"{"kind":"slack","webhook":"https://hooks.slack.com/x","channel":"#x"}"##;
        let ct = aead.encrypt(pt).unwrap();
        assert!(ct.contains("\"v\":1"));
        assert!(!ct.contains("hooks.slack.com")); // not in cleartext
        let back = aead.decrypt_or_plaintext(&ct).unwrap();
        assert_eq!(back, pt);
    }

    #[test]
    fn passthrough_without_key() {
        let aead = Aead::from_base64_or_passthrough(None).unwrap();
        assert!(!aead.is_active());
        let pt = r#"{"kind":"slack"}"#;
        assert_eq!(aead.encrypt(pt).unwrap(), pt);
        assert_eq!(aead.decrypt_or_plaintext(pt).unwrap(), pt);
    }

    #[test]
    fn reads_legacy_plaintext_when_key_present() {
        let aead = Aead::from_base64_or_passthrough(Some(&dev_key())).unwrap();
        let legacy = r#"{"kind":"telegram","bot_token":"t","chat_id":1}"#;
        assert_eq!(aead.decrypt_or_plaintext(legacy).unwrap(), legacy);
    }

    #[test]
    fn rejects_wrong_key() {
        let a = Aead::from_base64_or_passthrough(Some(&dev_key())).unwrap();
        let ct = a.encrypt("secret").unwrap();
        let other =
            Aead::from_base64_or_passthrough(Some(&BASE64_STANDARD.encode([9u8; 32]))).unwrap();
        assert!(other.decrypt_or_plaintext(&ct).is_err());
    }

    #[test]
    fn rejects_short_or_invalid_key() {
        assert!(Aead::from_base64_or_passthrough(Some("not_base64!!")).is_err());
        assert!(
            Aead::from_base64_or_passthrough(Some(&BASE64_STANDARD.encode([1u8; 16]))).is_err()
        );
    }
}
