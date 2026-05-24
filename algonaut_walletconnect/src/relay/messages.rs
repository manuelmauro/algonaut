//! WalletConnect v2 message types.
//!
//! This module defines the JSON-RPC message formats used by the
//! WalletConnect relay protocol.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub id: u64,
    pub jsonrpc: String,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    pub fn new(id: u64, method: &str, params: Option<Value>) -> Self {
        Self {
            id,
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
        }
    }
}

/// A JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub id: u64,
    pub jsonrpc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// A JSON-RPC 2.0 error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Relay protocol message envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayMessage {
    pub topic: String,
    pub message: String,
    #[serde(rename = "publishedAt")]
    pub published_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<u32>,
}

/// Relay subscription parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeParams {
    pub topic: String,
}

/// Relay publish parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishParams {
    pub topic: String,
    pub message: String,
    pub ttl: u64,
    pub tag: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<bool>,
}

/// WalletConnect pairing proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PairingProposal {
    pub id: u64,
    pub expiry: u64,
    pub relays: Vec<RelayProtocol>,
    pub proposer: Participant,
    #[serde(rename = "requiredNamespaces")]
    pub required_namespaces: RequiredNamespaces,
}

/// Relay protocol identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayProtocol {
    pub protocol: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

/// Session participant metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub metadata: AppMetadata,
}

/// Application metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMetadata {
    pub name: String,
    pub description: String,
    pub url: String,
    pub icons: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirect: Option<Redirect>,
}

/// Redirect configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Redirect {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub universal: Option<String>,
}

/// Required namespaces for session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct RequiredNamespaces {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub algorand: Option<NamespaceProposal>,
}

/// Namespace proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct NamespaceProposal {
    pub chains: Vec<String>,
    pub methods: Vec<String>,
    pub events: Vec<String>,
}

/// Session settle response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSettle {
    pub relay: RelayProtocol,
    pub namespaces: SessionNamespaces,
    pub controller: Participant,
    pub expiry: u64,
}

/// Session namespaces (response).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionNamespaces {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub algorand: Option<NamespaceResponse>,
}

/// Namespace response with accounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceResponse {
    pub accounts: Vec<String>,
    pub methods: Vec<String>,
    pub events: Vec<String>,
}

/// Algorand chain IDs.
#[allow(dead_code)]
pub mod chains {
    /// Algorand MainNet chain ID.
    pub const MAINNET: &str = "algorand:wGHE2Pwdvd7S12BL5FaOP20EGYesN73k";
    /// Algorand TestNet chain ID.
    pub const TESTNET: &str = "algorand:SGO1GKSzyE7IEPItTxCByw9x8FmnrCDe";
    /// Algorand BetaNet chain ID.
    pub const BETANET: &str = "algorand:mFgazF-2uRS1tMiL9dsj01hJGySEmPN2";
}

/// WalletConnect v2 JSON-RPC methods.
#[allow(dead_code)]
pub mod methods {
    /// Subscribe to a topic.
    pub const SUBSCRIBE: &str = "irn_subscribe";
    /// Unsubscribe from a topic.
    pub const UNSUBSCRIBE: &str = "irn_unsubscribe";
    /// Publish a message.
    pub const PUBLISH: &str = "irn_publish";
    /// Session proposal request.
    pub const SESSION_PROPOSE: &str = "wc_sessionPropose";
    /// Session settle response.
    pub const SESSION_SETTLE: &str = "wc_sessionSettle";
    /// Session request (e.g., algo_signTxn).
    pub const SESSION_REQUEST: &str = "wc_sessionRequest";
    /// Session delete.
    pub const SESSION_DELETE: &str = "wc_sessionDelete";
}

/// Message tags for routing.
#[allow(dead_code)]
pub mod tags {
    /// Session propose request.
    pub const SESSION_PROPOSE_REQUEST: u32 = 1100;
    /// Session propose response.
    pub const SESSION_PROPOSE_RESPONSE: u32 = 1101;
    /// Session settle request.
    pub const SESSION_SETTLE_REQUEST: u32 = 1102;
    /// Session settle response.
    pub const SESSION_SETTLE_RESPONSE: u32 = 1103;
    /// Session request.
    pub const SESSION_REQUEST: u32 = 1108;
    /// Session request response.
    pub const SESSION_REQUEST_RESPONSE: u32 = 1109;
}
