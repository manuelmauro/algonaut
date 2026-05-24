//! WalletConnect pairing URI handling.
//!
//! This module provides types for generating and parsing WalletConnect
//! pairing URIs used for QR codes and deep links.

use super::crypto::SymmetricKey;

/// A WalletConnect v2 pairing URI.
///
/// The URI format is:
/// `wc:{topic}@{version}?relay-protocol={protocol}&symKey={key}`
///
/// For Pera, this can be rendered as a QR code or opened as a deep link.
#[derive(Debug, Clone)]
pub struct PairingUri {
    /// The pairing topic (32 bytes hex-encoded).
    pub topic: String,
    /// The symmetric key for encrypting messages.
    pub sym_key: SymmetricKey,
    /// The relay protocol (usually "irn").
    pub relay_protocol: String,
}

impl PairingUri {
    /// Create a new pairing URI.
    pub fn new(topic: String, sym_key: SymmetricKey) -> Self {
        Self {
            topic,
            sym_key,
            relay_protocol: "irn".to_string(),
        }
    }

    /// Format as a WalletConnect URI string.
    pub fn to_uri(&self) -> String {
        format!(
            "wc:{}@2?relay-protocol={}&symKey={}",
            self.topic,
            self.relay_protocol,
            self.sym_key.to_hex()
        )
    }

    /// Generate a QR code as a string (ASCII art).
    pub fn to_qr_string(&self) -> String {
        use qrcode::{QrCode, render::unicode};

        let uri = self.to_uri();
        let code = QrCode::new(&uri).expect("URI should be valid for QR code");

        code.render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Light)
            .light_color(unicode::Dense1x2::Dark)
            .build()
    }

    /// Get the deep link URL for Pera Wallet.
    pub fn to_pera_deeplink(&self) -> String {
        let uri = self.to_uri();
        format!("perawallet-wc://wc?uri={}", urlencoding::encode(&uri))
    }
}

impl std::fmt::Display for PairingUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_uri())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay::crypto::generate_symmetric_key;

    #[test]
    fn test_pairing_uri_format() {
        let topic = "a".repeat(64);
        let sym_key = generate_symmetric_key();

        let uri = PairingUri::new(topic.clone(), sym_key.clone());
        let uri_string = uri.to_uri();

        assert!(uri_string.starts_with("wc:"));
        assert!(uri_string.contains("@2?"));
        assert!(uri_string.contains("relay-protocol=irn"));
        assert!(uri_string.contains(&format!("symKey={}", sym_key.to_hex())));
    }
}
