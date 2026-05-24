//! Cryptographic primitives for WalletConnect v2.
//!
//! This module provides:
//! - X25519 key exchange for deriving shared secrets
//! - ChaCha20-Poly1305 encryption/decryption for message envelopes
//! - HKDF key derivation

use chacha20poly1305::{
    ChaCha20Poly1305, KeyInit, Nonce,
    aead::{Aead, AeadCore, OsRng},
};
use hkdf::Hkdf;
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};

use super::error::RelayError;

/// A symmetric key derived from X25519 key exchange.
#[derive(Clone)]
pub struct SymmetricKey {
    key: [u8; 32],
}

impl SymmetricKey {
    /// Create a symmetric key from raw bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { key: bytes }
    }

    /// Create a symmetric key from a hex-encoded string.
    pub fn from_hex(hex: &str) -> Result<Self, RelayError> {
        let bytes = hex::decode(hex).map_err(|_| RelayError::InvalidKey)?;
        if bytes.len() != 32 {
            return Err(RelayError::InvalidKey);
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        Ok(Self { key })
    }

    /// Encode the key as a hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(self.key)
    }

    /// Encrypt a message using ChaCha20-Poly1305 with Type 0 envelope.
    ///
    /// Envelope format: type (1 byte) + nonce (12 bytes) + ciphertext + tag
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, RelayError> {
        let cipher =
            ChaCha20Poly1305::new_from_slice(&self.key).map_err(|_| RelayError::InvalidKey)?;

        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| RelayError::Encryption(e.to_string()))?;

        // Type 0 envelope: type byte + nonce + ciphertext
        let mut result = Vec::with_capacity(1 + 12 + ciphertext.len());
        result.push(0); // Type 0 envelope
        result.extend_from_slice(&nonce);
        result.extend(ciphertext);
        Ok(result)
    }

    /// Decrypt a message using ChaCha20-Poly1305 with Type 0 envelope.
    ///
    /// Envelope format: type (1 byte) + nonce (12 bytes) + ciphertext + tag
    pub fn decrypt(&self, envelope: &[u8]) -> Result<Vec<u8>, RelayError> {
        // Minimum: type (1) + nonce (12) + tag (16) = 29 bytes
        if envelope.len() < 29 {
            return Err(RelayError::Encryption("envelope too short".into()));
        }

        let envelope_type = envelope[0];
        if envelope_type != 0 {
            return Err(RelayError::Encryption(format!(
                "unsupported envelope type: {}",
                envelope_type
            )));
        }

        let cipher =
            ChaCha20Poly1305::new_from_slice(&self.key).map_err(|_| RelayError::InvalidKey)?;

        let nonce = Nonce::from_slice(&envelope[1..13]);
        let plaintext = cipher
            .decrypt(nonce, &envelope[13..])
            .map_err(|e| RelayError::Encryption(e.to_string()))?;

        Ok(plaintext)
    }
}

impl std::fmt::Debug for SymmetricKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SymmetricKey")
            .field("key", &"[redacted]")
            .finish()
    }
}

/// A key pair for X25519 key exchange.
pub struct KeyPair {
    secret: StaticSecret,
    public: PublicKey,
}

impl KeyPair {
    /// Generate a new random key pair.
    pub fn generate() -> Self {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        Self { secret, public }
    }

    /// Get the public key as bytes.
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.public.to_bytes()
    }

    /// Get the public key as a hex string.
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.public_key_bytes())
    }

    /// Perform X25519 key exchange and derive a symmetric key.
    pub fn derive_symmetric_key(&self, peer_public: &[u8; 32]) -> SymmetricKey {
        let peer_public = PublicKey::from(*peer_public);
        let shared_secret = self.secret.diffie_hellman(&peer_public);

        // Derive a 32-byte key using HKDF
        let hkdf = Hkdf::<Sha256>::new(None, shared_secret.as_bytes());
        let mut key = [0u8; 32];
        hkdf.expand(b"", &mut key)
            .expect("32 bytes is valid output length");

        SymmetricKey::from_bytes(key)
    }
}

impl std::fmt::Debug for KeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyPair")
            .field("public", &self.public_key_hex())
            .field("secret", &"[redacted]")
            .finish()
    }
}

/// Generate a random 32-byte topic.
pub fn generate_topic() -> String {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).expect("getrandom failed");
    hex::encode(bytes)
}

/// Generate a random symmetric key for pairing.
pub fn generate_symmetric_key() -> SymmetricKey {
    let mut key = [0u8; 32];
    getrandom::getrandom(&mut key).expect("getrandom failed");
    SymmetricKey::from_bytes(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symmetric_key_encrypt_decrypt() {
        let key = generate_symmetric_key();
        let plaintext = b"hello world";

        let ciphertext = key.encrypt(plaintext).unwrap();
        let decrypted = key.decrypt(&ciphertext).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_key_exchange() {
        let alice = KeyPair::generate();
        let bob = KeyPair::generate();

        let alice_key = alice.derive_symmetric_key(&bob.public_key_bytes());
        let bob_key = bob.derive_symmetric_key(&alice.public_key_bytes());

        // Both should derive the same key
        assert_eq!(alice_key.to_hex(), bob_key.to_hex());
    }
}
