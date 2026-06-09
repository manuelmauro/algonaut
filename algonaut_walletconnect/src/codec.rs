//! ARC-0001 codec for the `algo_signTxn` WalletConnect method.
//!
//! This module provides types for encoding signing requests and decoding
//! responses according to the [ARC-0001] specification.
//!
//! [ARC-0001]: https://arc.algorand.foundation/ARCs/arc-0001

use algonaut_core::{Address, ToMsgPack};
use algonaut_transaction::transaction::Transaction;
use data_encoding::BASE64;
use serde::{Deserialize, Serialize};

use crate::error::WalletConnectError;

/// A single transaction in the `algo_signTxn` request array.
///
/// Per ARC-0001, each element contains a base64-encoded unsigned transaction
/// and optional metadata for display and signing control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletTransaction {
    /// Base64-encoded canonical msgpack of the unsigned transaction.
    pub txn: String,

    /// Optional list of addresses that should sign this transaction.
    /// If present and non-empty, the wallet should sign.
    /// If present and empty (`[]`), this is display-only (the wallet should NOT sign).
    /// If absent, the wallet decides based on sender matching connected accounts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signers: Option<Vec<String>>,

    /// Optional message to display to the user for this transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Optional auth address for rekeyed accounts.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "authAddr")]
    pub auth_addr: Option<String>,

    /// Optional multisig metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msig: Option<MultisigMetadata>,

    /// Optional group message displayed once for the entire group.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "groupMessage")]
    pub group_message: Option<String>,
}

/// Multisig metadata for ARC-0001 signing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigMetadata {
    /// Multisig version.
    pub version: u8,
    /// Number of signatures required.
    pub threshold: u8,
    /// List of base64-encoded public keys.
    #[serde(rename = "addrs")]
    pub addresses: Vec<String>,
}

/// A signing request containing the full group context and which indexes
/// should be signed by this wallet.
#[derive(Debug, Clone)]
pub struct SignRequest {
    /// The full atomic group of transactions.
    pub transactions: Vec<Transaction>,
    /// Indexes into `transactions` that this signer should sign.
    pub indexes_to_sign: Vec<usize>,
    /// The connected wallet address that will sign.
    pub signer_address: Address,
    /// Optional message to display for the entire group.
    pub group_message: Option<String>,
}

impl SignRequest {
    /// Encode this request into the ARC-0001 `WalletTransaction[]` format.
    ///
    /// Transactions at `indexes_to_sign` get `signers: [signer_address]`,
    /// all others get `signers: []` (display-only).
    pub fn encode(&self) -> Result<Vec<WalletTransaction>, WalletConnectError> {
        self.transactions
            .iter()
            .enumerate()
            .map(|(i, tx)| {
                let txn_bytes = tx
                    .to_msg_pack()
                    .map_err(|e| WalletConnectError::EncodingError(e.to_string()))?;
                let txn = BASE64.encode(&txn_bytes);

                let signers = if self.indexes_to_sign.contains(&i) {
                    // This signer owns this slot
                    Some(vec![self.signer_address.to_string()])
                } else {
                    // Display-only: empty signers array
                    Some(vec![])
                };

                let group_message = if i == 0 {
                    self.group_message.clone()
                } else {
                    None
                };

                Ok(WalletTransaction {
                    txn,
                    signers,
                    message: None,
                    auth_addr: None,
                    msig: None,
                    group_message,
                })
            })
            .collect()
    }
}

/// Response element from `algo_signTxn`.
///
/// The wallet returns an array where:
/// - Signed transactions are base64-encoded signed transaction msgpack
/// - Display-only slots are `null`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SignedTxnResponse {
    /// A signed transaction (base64-encoded msgpack).
    Signed(String),
    /// A display-only slot that wasn't signed.
    Null,
}

impl SignedTxnResponse {
    /// Returns the base64 string if this is a signed transaction.
    pub fn as_signed(&self) -> Option<&str> {
        match self {
            SignedTxnResponse::Signed(s) => Some(s),
            SignedTxnResponse::Null => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use algonaut_core::{MicroAlgos, Round};
    use algonaut_crypto::HashDigest;
    use algonaut_model::algod::SuggestedParams;
    use algonaut_transaction::Pay;
    use algonaut_transaction::account::Account;

    fn mock_params() -> SuggestedParams {
        SuggestedParams {
            genesis_id: "testnet-v1.0".to_string(),
            genesis_hash: HashDigest([0u8; 32]),
            consensus_version: "".to_string(),
            fee: MicroAlgos(0),
            min_fee: MicroAlgos(1000),
            last_round: Round(1000),
        }
    }

    #[test]
    fn test_encode_single_signer() {
        let sender = Account::generate();
        let receiver = Account::generate();

        let params = mock_params();
        let tx = Pay::new(sender.address(), receiver.address(), MicroAlgos(1_000_000))
            .build(&params)
            .unwrap();

        let request = SignRequest {
            transactions: vec![tx],
            indexes_to_sign: vec![0],
            signer_address: sender.address(),
            group_message: Some("Test payment".to_string()),
        };

        let encoded = request.encode().unwrap();
        assert_eq!(encoded.len(), 1);
        assert_eq!(encoded[0].signers, Some(vec![sender.address().to_string()]));
        assert_eq!(encoded[0].group_message, Some("Test payment".to_string()));
    }

    #[test]
    fn test_encode_partial_signer() {
        let alice = Account::generate();
        let bob = Account::generate();

        let params = mock_params();
        let tx1 = Pay::new(alice.address(), bob.address(), MicroAlgos(1_000_000))
            .build(&params)
            .unwrap();
        let tx2 = Pay::new(bob.address(), alice.address(), MicroAlgos(500_000))
            .build(&params)
            .unwrap();

        // Alice only signs index 0
        let request = SignRequest {
            transactions: vec![tx1, tx2],
            indexes_to_sign: vec![0],
            signer_address: alice.address(),
            group_message: None,
        };

        let encoded = request.encode().unwrap();
        assert_eq!(encoded.len(), 2);
        // Alice signs index 0
        assert_eq!(encoded[0].signers, Some(vec![alice.address().to_string()]));
        // Index 1 is display-only
        assert_eq!(encoded[1].signers, Some(vec![]));
    }
}
