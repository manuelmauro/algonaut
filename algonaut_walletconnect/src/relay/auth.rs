//! WalletConnect relay authentication.
//!
//! This module handles JWT generation for authenticating with the
//! WalletConnect relay server.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::{SecretKey, Signer, SigningKey};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

use super::error::RelayError;

/// An identity key pair for relay authentication.
///
/// This Ed25519 key pair is used to sign JWTs for authenticating
/// with the WalletConnect relay server.
pub struct IdentityKey {
    signing_key: SigningKey,
}

impl IdentityKey {
    /// Generate a new random identity key.
    pub fn generate() -> Self {
        let mut secret_bytes: SecretKey = [0u8; 32];
        getrandom::getrandom(&mut secret_bytes).expect("getrandom failed");
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        Self { signing_key }
    }

    /// Get the client ID (did:key encoded public key).
    pub fn client_id(&self) -> String {
        let public_key = self.signing_key.verifying_key();
        let public_key_bytes = public_key.to_bytes();

        // Encode as did:key with multicodec prefix for Ed25519 (0xed01)
        let mut multicodec = vec![0xed, 0x01];
        multicodec.extend_from_slice(&public_key_bytes);

        // Use multibase base58btc encoding (prefix 'z')
        let encoded = bs58::encode(&multicodec).into_string();
        format!("did:key:z{}", encoded)
    }

    /// Generate a signed JWT for relay authentication.
    ///
    /// # Arguments
    ///
    /// * `aud` - The relay server URL (audience)
    pub fn generate_jwt(&self, aud: &str) -> Result<String, RelayError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| RelayError::Encryption(e.to_string()))?
            .as_secs();

        // Generate random subject/nonce
        let mut sub_bytes = [0u8; 32];
        getrandom::getrandom(&mut sub_bytes).map_err(|e| RelayError::Encryption(e.to_string()))?;
        let sub = hex::encode(sub_bytes);

        // JWT Header
        let header = json!({
            "alg": "EdDSA",
            "typ": "JWT"
        });

        // JWT Payload
        let payload = json!({
            "iat": now,
            "exp": now + 86400, // 24 hours
            "iss": self.client_id(),
            "aud": aud,
            "sub": sub,
            "act": "client_auth"
        });

        // Encode header and payload
        let header_b64 = URL_SAFE_NO_PAD.encode(header.to_string().as_bytes());
        let payload_b64 = URL_SAFE_NO_PAD.encode(payload.to_string().as_bytes());
        let message = format!("{}.{}", header_b64, payload_b64);

        // Sign the message
        let signature = self.signing_key.sign(message.as_bytes());
        let signature_b64 = URL_SAFE_NO_PAD.encode(signature.to_bytes());

        Ok(format!("{}.{}", message, signature_b64))
    }
}

impl std::fmt::Debug for IdentityKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IdentityKey")
            .field("client_id", &self.client_id())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_id_format() {
        let key = IdentityKey::generate();
        let client_id = key.client_id();

        assert!(client_id.starts_with("did:key:z"));
    }

    #[test]
    fn test_jwt_format() {
        let key = IdentityKey::generate();
        let jwt = key.generate_jwt("wss://relay.walletconnect.com").unwrap();

        // JWT should have 3 parts separated by dots
        let parts: Vec<&str> = jwt.split('.').collect();
        assert_eq!(parts.len(), 3);
    }
}
