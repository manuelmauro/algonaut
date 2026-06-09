//! WalletConnect session state management.

use algonaut_core::Address;

use super::crypto::SymmetricKey;
use super::error::RelayError;
use super::messages::{AppMetadata, SessionNamespaces};

/// The state of a WalletConnect session.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum SessionState {
    /// Initial state, waiting for pairing.
    Pending,
    /// Pairing completed, session proposal sent.
    Proposed {
        /// The session topic (sha256 of the session symmetric key).
        session_topic: String,
        /// The session symmetric key for decryption.
        session_sym_key: SymmetricKey,
    },
    /// Session established and active.
    Active(Box<ActiveSession>),
    /// Session has been closed.
    Closed,
}

/// An active WalletConnect session.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ActiveSession {
    /// The session topic for encrypted communication.
    pub topic: String,
    /// The symmetric key for this session.
    pub sym_key: SymmetricKey,
    /// The connected account addresses.
    pub accounts: Vec<String>,
    /// The wallet's metadata.
    pub peer_metadata: Option<AppMetadata>,
    /// Session expiry timestamp (Unix seconds).
    pub expiry: u64,
    /// Namespaces approved by the wallet.
    pub namespaces: SessionNamespaces,
}

impl ActiveSession {
    /// Get the first Algorand address from the session.
    ///
    /// Account format is `algorand:{chain_reference}:{address}`.
    pub fn algorand_address(&self) -> Result<Address, RelayError> {
        let account = self.accounts.first().ok_or(RelayError::NoAccounts)?;

        // Parse "algorand:{chain}:{address}" format
        let parts: Vec<&str> = account.split(':').collect();
        if parts.len() != 3 || parts[0] != "algorand" {
            return Err(RelayError::InvalidAccount(account.clone()));
        }

        parts[2]
            .parse()
            .map_err(|_| RelayError::InvalidAccount(account.clone()))
    }

    /// Check if the session is still valid (not expired).
    pub fn is_valid(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now < self.expiry
    }
}

/// Session proposal configuration.
#[derive(Debug, Clone)]
pub struct SessionProposalConfig {
    /// The dApp metadata to send to the wallet.
    pub metadata: AppMetadata,
    /// The Algorand chain IDs to request.
    pub chains: Vec<String>,
    /// The RPC methods to request.
    pub methods: Vec<String>,
    /// The events to subscribe to.
    pub events: Vec<String>,
}

impl Default for SessionProposalConfig {
    fn default() -> Self {
        Self {
            metadata: AppMetadata {
                name: "algonaut".to_string(),
                description: "Algorand Rust SDK".to_string(),
                url: "https://github.com/manuelmauro/algonaut".to_string(),
                icons: vec![],
                redirect: None,
            },
            // A single chain only: with requiredNamespaces the wallet must
            // satisfy every listed chain, and wallets (e.g. Pera) reject a
            // proposal that requires both MainNet and TestNet at once. Use
            // `testnet()` / `with_chains()` to target a different network.
            chains: vec![super::messages::chains::MAINNET.to_string()],
            methods: vec!["algo_signTxn".to_string()],
            events: vec![],
        }
    }
}

impl SessionProposalConfig {
    /// Create a config for MainNet only.
    pub fn mainnet() -> Self {
        Self {
            chains: vec![super::messages::chains::MAINNET.to_string()],
            ..Default::default()
        }
    }

    /// Create a config for TestNet only.
    pub fn testnet() -> Self {
        Self {
            chains: vec![super::messages::chains::TESTNET.to_string()],
            ..Default::default()
        }
    }

    /// Set the dApp metadata.
    pub fn with_metadata(mut self, metadata: AppMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}
