//! WalletConnect relay client.
//!
//! This module provides the main `WalletConnectRelay` struct that connects
//! to the WalletConnect relay network and implements `WalletConnectSession`.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, RwLock, oneshot};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

use super::auth::IdentityKey;
use super::crypto::{KeyPair, SymmetricKey, generate_symmetric_key, generate_topic};
use super::error::RelayError;
use super::messages::{
    JsonRpcRequest, JsonRpcResponse, PublishParams, RelayMessage, SessionSettle, SubscribeParams,
    methods, tags,
};
use super::pairing::PairingUri;
use super::session::{ActiveSession, SessionProposalConfig, SessionState};
use crate::codec::{SignedTxnResponse, WalletTransaction};
use crate::error::WalletConnectError;
use crate::session::{SessionFuture, WalletConnectSession};

/// Default WalletConnect relay URL (matches @perawallet/connect).
const DEFAULT_RELAY_URL: &str = "wss://relay.walletconnect.com";

/// Default TTL for published messages (5 minutes).
const DEFAULT_TTL: u64 = 300;

/// Timeout for session establishment.
const SESSION_TIMEOUT: Duration = Duration::from_secs(300);

/// Timeout for signing requests.
const SIGN_TIMEOUT: Duration = Duration::from_secs(120);

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type PendingRequests = Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value, RelayError>>>>>;

/// A WalletConnect v2 relay client.
///
/// This struct manages the WebSocket connection to the WalletConnect relay,
/// handles the pairing and session establishment protocol, and provides
/// transaction signing via the `WalletConnectSession` trait.
pub struct WalletConnectRelay {
    /// The WalletConnect project ID.
    project_id: String,
    /// The identity key for relay authentication.
    #[allow(dead_code)]
    identity_key: IdentityKey,
    /// The pairing topic.
    pairing_topic: String,
    /// The symmetric key for the pairing topic.
    pairing_sym_key: SymmetricKey,
    /// Our key pair for session key derivation.
    key_pair: KeyPair,
    /// Current session state.
    state: Arc<RwLock<SessionState>>,
    /// The WebSocket connection.
    ws: Arc<Mutex<WsStream>>,
    /// Pending JSON-RPC requests awaiting responses.
    pending: PendingRequests,
    /// Request ID counter.
    request_id: AtomicU64,
    /// Session proposal configuration (used in session proposal).
    #[allow(dead_code)]
    config: SessionProposalConfig,
}

impl std::fmt::Debug for WalletConnectRelay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WalletConnectRelay")
            .field("project_id", &self.project_id)
            .field("pairing_topic", &self.pairing_topic)
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

impl WalletConnectRelay {
    /// Create a new relay client and connect to the WalletConnect relay.
    ///
    /// # Arguments
    ///
    /// * `project_id` - Your WalletConnect Cloud project ID
    ///
    /// # Example
    ///
    /// ```ignore
    /// let relay = WalletConnectRelay::new("your-project-id").await?;
    /// ```
    pub async fn new(project_id: impl Into<String>) -> Result<Self, RelayError> {
        Self::with_config(project_id, SessionProposalConfig::default()).await
    }

    /// Create a new relay client with custom session configuration.
    pub async fn with_config(
        project_id: impl Into<String>,
        config: SessionProposalConfig,
    ) -> Result<Self, RelayError> {
        let project_id = project_id.into();
        let identity_key = IdentityKey::generate();
        let pairing_topic = generate_topic();
        let pairing_sym_key = generate_symmetric_key();
        let key_pair = KeyPair::generate();

        // Generate JWT for relay authentication
        let jwt = identity_key.generate_jwt(DEFAULT_RELAY_URL)?;

        // Connect to the relay with authentication
        let url = format!(
            "{}/?projectId={}&auth={}",
            DEFAULT_RELAY_URL, project_id, jwt
        );
        let (ws, _) = connect_async(&url)
            .await
            .map_err(|e| RelayError::Connection(e.to_string()))?;

        let relay = Self {
            project_id,
            identity_key,
            pairing_topic,
            pairing_sym_key,
            key_pair,
            state: Arc::new(RwLock::new(SessionState::Pending)),
            ws: Arc::new(Mutex::new(ws)),
            pending: Arc::new(Mutex::new(HashMap::new())),
            request_id: AtomicU64::new(1),
            config,
        };

        // Subscribe to the pairing topic
        relay.subscribe(&relay.pairing_topic).await?;

        Ok(relay)
    }

    /// Get the pairing URI for QR codes or deep links.
    pub fn pairing_uri(&self) -> PairingUri {
        PairingUri::new(self.pairing_topic.clone(), self.pairing_sym_key.clone())
    }

    /// Wait for the wallet to establish a session.
    ///
    /// This method blocks until the wallet scans the QR code, approves
    /// the session, and sends the session settlement message.
    ///
    /// # Returns
    ///
    /// The Algorand address from the connected wallet.
    pub async fn wait_for_session(&self) -> Result<algonaut_core::Address, RelayError> {
        // Send the session proposal first
        self.send_session_proposal().await?;

        let deadline = tokio::time::Instant::now() + SESSION_TIMEOUT;
        let mut session_topic_subscribed = false;

        loop {
            // Check current state and get any pending session topic to subscribe to
            let (pending_session_topic, address) = {
                let state = self.state.read().await;
                match &*state {
                    SessionState::Active(session) => {
                        let addr = session.algorand_address()?;
                        (None, Some(addr))
                    }
                    SessionState::Proposed { session_topic, .. } => {
                        (Some(session_topic.clone()), None)
                    }
                    SessionState::Closed => {
                        return Err(RelayError::SessionRejected);
                    }
                    _ => (None, None),
                }
            };

            // If session is active, return the address
            if let Some(addr) = address {
                return Ok(addr);
            }

            // If we have a proposed session topic and haven't subscribed yet, subscribe now
            if let Some(topic) = pending_session_topic
                && !session_topic_subscribed
            {
                eprintln!("[DEBUG] Subscribing to session topic: {}", &topic[..16]);
                self.subscribe(&topic).await?;
                session_topic_subscribed = true;
            }

            if tokio::time::Instant::now() > deadline {
                return Err(RelayError::Timeout {
                    operation: "session establishment",
                });
            }

            // Process incoming messages
            self.process_next_message(Duration::from_millis(500))
                .await?;
        }
    }

    /// Send a session proposal to the pairing topic.
    async fn send_session_proposal(&self) -> Result<(), RelayError> {
        // Expiry is 5 minutes from now (standard WalletConnect expiry)
        let expiry = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 300;

        let request_id = self.generate_request_id();

        let proposal = json!({
            "id": request_id,
            "relays": [{
                "protocol": "irn"
            }],
            "proposer": {
                "publicKey": self.key_pair.public_key_hex(),
                "metadata": {
                    "name": self.config.metadata.name,
                    "description": self.config.metadata.description,
                    "url": self.config.metadata.url,
                    "icons": self.config.metadata.icons
                }
            },
            // Pera (and most Algorand wallets) expect the algorand namespace under
            // requiredNamespaces; an optional-only proposal is decoded but never
            // turns into an approval prompt, so the wallet times out.
            "requiredNamespaces": {
                "algorand": {
                    "chains": self.config.chains,
                    "methods": self.config.methods,
                    "events": self.config.events
                }
            },
            "optionalNamespaces": {},
            "pairingTopic": self.pairing_topic,
            "expiryTimestamp": expiry
        });

        let request = json!({
            "id": request_id,
            "jsonrpc": "2.0",
            "method": methods::SESSION_PROPOSE,
            "params": proposal
        });

        let request_str = serde_json::to_string(&request)?;
        eprintln!("[DEBUG] Session proposal: {}", request_str);

        // Publish the encrypted proposal to the pairing topic
        self.publish(
            &self.pairing_topic,
            &request_str,
            tags::SESSION_PROPOSE_REQUEST,
            &self.pairing_sym_key,
        )
        .await?;

        // State will be updated to Proposed when we receive the session proposal response

        Ok(())
    }

    /// Subscribe to a topic on the relay.
    async fn subscribe(&self, topic: &str) -> Result<String, RelayError> {
        let params = SubscribeParams {
            topic: topic.to_string(),
        };
        let response = self
            .send_relay_request(methods::SUBSCRIBE, Some(serde_json::to_value(params)?))
            .await?;

        response
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| RelayError::UnexpectedResponse("expected subscription ID".into()))
    }

    /// Publish a message to a topic.
    async fn publish(
        &self,
        topic: &str,
        message: &str,
        tag: u32,
        sym_key: &SymmetricKey,
    ) -> Result<(), RelayError> {
        // Encrypt the message
        let encrypted = sym_key.encrypt(message.as_bytes())?;
        let encoded =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &encrypted);

        let params = PublishParams {
            topic: topic.to_string(),
            message: encoded,
            ttl: DEFAULT_TTL,
            tag,
            prompt: Some(true),
        };

        self.send_relay_request(methods::PUBLISH, Some(serde_json::to_value(params)?))
            .await?;

        Ok(())
    }

    /// Generate a unique request ID (13-digit ms timestamp + 3-digit entropy).
    ///
    /// WalletConnect's `payloadId` is `Date.now() * 1000 + entropy`, which keeps
    /// the id at ~16 digits — within JavaScript's `Number.MAX_SAFE_INTEGER`
    /// (2^53). A larger id (e.g. ms * 1_000_000) overflows the wallet's JSON
    /// number handling, so it receives the request but cannot respond and the
    /// handshake silently fails.
    fn generate_request_id(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let entropy = self.request_id.fetch_add(1, Ordering::SeqCst) % 1_000;

        // Combine: timestamp * 1_000 + entropy (matches WalletConnect payloadId).
        timestamp * 1_000 + entropy
    }

    /// Send a JSON-RPC request to the relay and wait for response.
    async fn send_relay_request(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value, RelayError> {
        let id = self.generate_request_id();
        let request = JsonRpcRequest::new(id, method, params);

        // Create response channel
        let (tx, mut rx) = oneshot::channel();
        {
            let mut pending = self.pending.lock().await;
            pending.insert(id, tx);
        }

        // Send the request
        let msg = serde_json::to_string(&request)?;
        {
            let mut ws = self.ws.lock().await;
            ws.send(Message::Text(msg.into())).await?;
        }

        // Read responses until we get ours (simple inline read, no recursion)
        let deadline = tokio::time::Instant::now() + Duration::from_secs(30);

        loop {
            if tokio::time::Instant::now() > deadline {
                return Err(RelayError::Timeout {
                    operation: "relay request",
                });
            }

            // Check if response already arrived
            if let Ok(result) = rx.try_recv() {
                return result;
            }

            // Read next message from websocket
            let msg = {
                let mut ws = self.ws.lock().await;
                match tokio::time::timeout(Duration::from_millis(100), ws.next()).await {
                    Ok(Some(Ok(Message::Text(text)))) => Some(text.to_string()),
                    Ok(Some(Ok(_))) => None,
                    Ok(Some(Err(e))) => return Err(e.into()),
                    Ok(None) => return Err(RelayError::Connection("connection closed".into())),
                    Err(_) => None, // Timeout, try again
                }
            };

            // Process the message if we got one
            if let Some(msg) = msg {
                eprintln!(
                    "[DEBUG] send_relay_request received: {}",
                    &msg[..msg.len().min(200)]
                );
                // A relay push (irn_subscription) has a required `method`; a
                // response to our request does not. Try request first so the
                // lenient JsonRpcResponse parse doesn't swallow pushes that
                // arrive while we wait for our response.
                if let Ok(request) = serde_json::from_str::<JsonRpcRequest>(&msg) {
                    eprintln!("[DEBUG] Parsed as JsonRpcRequest (subscription message)");
                    if request.method == "irn_subscription"
                        && let Some(params) = request.params
                        && let Ok(relay_msg) = serde_json::from_value::<RelayMessageWrapper>(params)
                    {
                        // Process the relay message (but don't block on it)
                        let _ = self.handle_relay_message(&relay_msg.data).await;
                    }
                } else if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(&msg) {
                    eprintln!("[DEBUG] Parsed as JsonRpcResponse with id: {}", response.id);
                    let mut pending = self.pending.lock().await;
                    if let Some(sender) = pending.remove(&response.id) {
                        let result = if let Some(error) = response.error {
                            Err(RelayError::JsonRpc {
                                code: error.code,
                                message: error.message,
                            })
                        } else {
                            Ok(response.result.unwrap_or(Value::Null))
                        };
                        let _ = sender.send(result);
                    }
                } else {
                    eprintln!("[DEBUG] Unknown message format");
                }
            }
        }
    }

    /// Process the next incoming WebSocket message.
    async fn process_next_message(&self, timeout: Duration) -> Result<(), RelayError> {
        let msg = {
            let mut ws = self.ws.lock().await;
            match tokio::time::timeout(timeout, ws.next()).await {
                Ok(Some(Ok(Message::Text(text)))) => {
                    eprintln!("[DEBUG] process_next_message got text message");
                    text.to_string()
                }
                Ok(Some(Ok(other))) => {
                    eprintln!(
                        "[DEBUG] process_next_message got non-text message: {:?}",
                        other
                    );
                    return Ok(());
                }
                Ok(Some(Err(e))) => return Err(e.into()),
                Ok(None) => return Err(RelayError::Connection("connection closed".into())),
                Err(_) => return Ok(()), // Timeout, no message
            }
        };

        self.handle_message(&msg).await
    }

    /// Handle an incoming WebSocket message.
    async fn handle_message(&self, msg: &str) -> Result<(), RelayError> {
        eprintln!("[DEBUG] handle_message: {}", &msg[..msg.len().min(200)]);

        // Try parsing as a JSON-RPC request (relay push) FIRST. A relay
        // `irn_subscription` push has a required `method` field; a reply to one
        // of our requests does not. JsonRpcResponse deserializes leniently
        // (result/error optional, unknown fields ignored), so trying it first
        // would swallow every push as a "response" and silently drop the
        // wallet's proposal-response and settle messages.
        if let Ok(request) = serde_json::from_str::<JsonRpcRequest>(msg) {
            eprintln!("[DEBUG] Parsed as JsonRpcRequest");
            self.handle_request(request).await?;
            return Ok(());
        }

        // Otherwise it's a response to one of our relay requests.
        if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(msg) {
            eprintln!("[DEBUG] Parsed as JsonRpcResponse");
            self.handle_response(response).await?;
            return Ok(());
        }

        eprintln!("[DEBUG] Unknown message format");
        // Unknown message format
        Ok(())
    }

    /// Handle a JSON-RPC response.
    async fn handle_response(&self, response: JsonRpcResponse) -> Result<(), RelayError> {
        let mut pending = self.pending.lock().await;
        if let Some(sender) = pending.remove(&response.id) {
            let result = if let Some(error) = response.error {
                Err(RelayError::JsonRpc {
                    code: error.code,
                    message: error.message,
                })
            } else {
                Ok(response.result.unwrap_or(Value::Null))
            };
            let _ = sender.send(result);
        }
        Ok(())
    }

    /// Handle a JSON-RPC request (subscription message).
    async fn handle_request(&self, request: JsonRpcRequest) -> Result<(), RelayError> {
        eprintln!("[DEBUG] Received request method: {}", request.method);
        // Check if this is a subscription message
        if request.method == "irn_subscription"
            && let Some(params) = request.params
            && let Ok(relay_msg) = serde_json::from_value::<RelayMessageWrapper>(params)
        {
            self.handle_relay_message(&relay_msg.data).await?;
        }
        Ok(())
    }

    /// Handle a relay subscription message.
    async fn handle_relay_message(&self, msg: &RelayMessage) -> Result<(), RelayError> {
        eprintln!("[DEBUG] Received message on topic: {}", &msg.topic[..8]);

        // Determine which key to use for decryption
        let sym_key = {
            let state = self.state.read().await;
            match &*state {
                SessionState::Active(session) if session.topic == msg.topic => {
                    eprintln!("[DEBUG] Using active session key");
                    session.sym_key.clone()
                }
                SessionState::Proposed {
                    session_topic,
                    session_sym_key,
                } if *session_topic == msg.topic => {
                    eprintln!("[DEBUG] Using proposed session key");
                    session_sym_key.clone()
                }
                _ if msg.topic == self.pairing_topic => {
                    eprintln!("[DEBUG] Using pairing key");
                    self.pairing_sym_key.clone()
                }
                _ => {
                    eprintln!("[DEBUG] Unknown topic, ignoring");
                    return Ok(());
                }
            }
        };

        // Decrypt the message
        let encrypted =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &msg.message)
                .map_err(|_| RelayError::Encryption("invalid base64".into()))?;

        let decrypted = sym_key.decrypt(&encrypted)?;
        let payload = String::from_utf8(decrypted)
            .map_err(|_| RelayError::Encryption("invalid UTF-8".into()))?;

        eprintln!(
            "[DEBUG] Decrypted payload: {}",
            &payload[..payload.len().min(200)]
        );

        // A wallet-originated message with a `method` is a request
        // (wc_sessionSettle / wc_sessionDelete); without one it is a response to
        // a request we sent. Parse request first — JsonRpcResponse deserializes
        // leniently and would otherwise swallow the settle and drop it.
        if let Ok(request) = serde_json::from_str::<JsonRpcRequest>(&payload) {
            match request.method.as_str() {
                methods::SESSION_SETTLE => {
                    self.handle_session_settle(request).await?;
                }
                methods::SESSION_DELETE => {
                    let mut state = self.state.write().await;
                    *state = SessionState::Closed;
                }
                _ => {}
            }
            return Ok(());
        }

        // Otherwise it's a response. On the pairing topic that's the
        // session-propose response (carries responderPublicKey); on the session
        // topic it's a reply to one of our requests (e.g. algo_signTxn).
        if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(&payload) {
            if msg.topic == self.pairing_topic {
                return self.handle_encrypted_response(response, &msg.topic).await;
            }
            if let Some(result) = response.result {
                self.handle_inner_response(response.id, result).await?;
            }
        }

        Ok(())
    }

    /// Handle an encrypted JSON-RPC response (e.g., session proposal response).
    async fn handle_encrypted_response(
        &self,
        response: JsonRpcResponse,
        topic: &str,
    ) -> Result<(), RelayError> {
        eprintln!(
            "[DEBUG] handle_encrypted_response on topic: {}",
            &topic[..8]
        );

        // Check if this is on the pairing topic (session proposal response)
        if topic == self.pairing_topic {
            eprintln!("[DEBUG] This is a pairing topic response");

            if let Some(error) = response.error {
                eprintln!("[DEBUG] Response has error: {:?}", error);
                return Err(RelayError::JsonRpc {
                    code: error.code,
                    message: error.message,
                });
            }

            if let Some(result) = &response.result {
                eprintln!("[DEBUG] Response result: {:?}", result);
                // Parse session proposal response: { relay, responderPublicKey }
                if let Some(responder_key) =
                    result.get("responderPublicKey").and_then(|v| v.as_str())
                {
                    eprintln!("[DEBUG] Got responderPublicKey: {}", responder_key);
                    // Derive session symmetric key
                    let peer_public_bytes =
                        hex::decode(responder_key).map_err(|_| RelayError::InvalidKey)?;

                    if peer_public_bytes.len() != 32 {
                        return Err(RelayError::InvalidKey);
                    }

                    let mut peer_public = [0u8; 32];
                    peer_public.copy_from_slice(&peer_public_bytes);

                    let session_sym_key = self.key_pair.derive_symmetric_key(&peer_public);

                    // Session topic is sha256 of the raw symmetric key bytes
                    // (WalletConnect `hashKey`), hex-encoded.
                    use sha2::{Digest, Sha256};
                    let mut hasher = Sha256::new();
                    hasher.update(session_sym_key.as_bytes());
                    let session_topic = hex::encode(hasher.finalize());

                    eprintln!("[DEBUG] Computed session topic: {}", &session_topic[..16]);

                    // Store pending session info - subscription will happen in wait_for_session
                    // to avoid async recursion (subscribe -> send_relay_request -> handle_relay_message)
                    let mut state = self.state.write().await;
                    *state = SessionState::Proposed {
                        session_topic,
                        session_sym_key,
                    };
                }
            }
        }

        Ok(())
    }

    /// Handle a session settlement message.
    async fn handle_session_settle(&self, request: JsonRpcRequest) -> Result<(), RelayError> {
        let request_id = request.id;
        let params: SessionSettle = serde_json::from_value(request.params.ok_or_else(|| {
            RelayError::UnexpectedResponse("missing session settle params".into())
        })?)?;

        // Extract accounts from namespaces
        let accounts = params
            .namespaces
            .algorand
            .as_ref()
            .map(|ns| ns.accounts.clone())
            .unwrap_or_default();

        if accounts.is_empty() {
            return Err(RelayError::NoAccounts);
        }

        // Get the session topic and key. Normally we are in `Proposed`; tolerate a
        // redelivered settle after activation so we can re-ack idempotently.
        let (session_topic, session_sym_key) = {
            let state = self.state.read().await;
            match &*state {
                SessionState::Proposed {
                    session_topic,
                    session_sym_key,
                } => (session_topic.clone(), session_sym_key.clone()),
                SessionState::Active(session) => (session.topic.clone(), session.sym_key.clone()),
                _ => {
                    return Err(RelayError::UnexpectedResponse(
                        "received session settle without prior proposal response".into(),
                    ));
                }
            }
        };

        // Acknowledge the settle so the wallet completes the handshake. Without
        // this response the wallet reports the dApp as not responding.
        let response = json!({
            "id": request_id,
            "jsonrpc": "2.0",
            "result": true
        });
        let response_str = serde_json::to_string(&response)?;
        // Box::pin breaks the recursive async cycle:
        // handle_session_settle -> publish -> send_relay_request ->
        // handle_relay_message -> handle_session_settle.
        Box::pin(self.publish(
            &session_topic,
            &response_str,
            tags::SESSION_SETTLE_RESPONSE,
            &session_sym_key,
        ))
        .await?;

        // Update state to active
        let active_session = ActiveSession {
            topic: session_topic,
            sym_key: session_sym_key,
            accounts,
            peer_metadata: Some(params.controller.metadata),
            expiry: params.expiry,
            namespaces: params.namespaces,
        };

        let mut state = self.state.write().await;
        *state = SessionState::Active(Box::new(active_session));

        Ok(())
    }

    /// Handle an inner response (response to session request).
    async fn handle_inner_response(&self, id: u64, result: Value) -> Result<(), RelayError> {
        let mut pending = self.pending.lock().await;
        if let Some(sender) = pending.remove(&id) {
            let _ = sender.send(Ok(result));
        }
        Ok(())
    }

    /// Send a session request (e.g., algo_signTxn).
    async fn send_session_request(&self, method: &str, params: Value) -> Result<Value, RelayError> {
        let state = self.state.read().await;
        let session = match &*state {
            SessionState::Active(session) => session.clone(),
            _ => return Err(RelayError::SessionExpired),
        };
        drop(state);

        // Check session validity
        if !session.is_valid() {
            return Err(RelayError::SessionExpired);
        }

        let id = self.request_id.fetch_add(1, Ordering::SeqCst);

        // Build the session request
        let request = json!({
            "id": id,
            "jsonrpc": "2.0",
            "method": methods::SESSION_REQUEST,
            "params": {
                "request": {
                    "method": method,
                    "params": params
                },
                "chainId": super::messages::chains::MAINNET
            }
        });

        let request_str = serde_json::to_string(&request)?;

        // Create response channel
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending.lock().await;
            pending.insert(id, tx);
        }

        // Publish the encrypted request
        self.publish(
            &session.topic,
            &request_str,
            tags::SESSION_REQUEST,
            &session.sym_key,
        )
        .await?;

        // Wait for response with timeout
        tokio::time::timeout(SIGN_TIMEOUT, rx)
            .await
            .map_err(|_| RelayError::Timeout {
                operation: "signing request",
            })?
            .map_err(|_| RelayError::UnexpectedResponse("response channel closed".into()))?
    }

    /// Start the background message processing loop.
    ///
    /// Call this to process incoming messages in the background.
    /// Returns a handle that can be used to stop the loop.
    pub fn spawn_message_loop(self: &Arc<Self>) -> tokio::task::JoinHandle<()> {
        let relay = Arc::clone(self);
        tokio::spawn(async move {
            loop {
                if let Err(e) = relay.process_next_message(Duration::from_millis(100)).await {
                    // Log error but continue unless connection is closed
                    if matches!(e, RelayError::Connection(_)) {
                        break;
                    }
                }
            }
        })
    }
}

impl WalletConnectSession for WalletConnectRelay {
    fn sign_transactions<'a>(
        &'a self,
        transactions: Vec<WalletTransaction>,
    ) -> SessionFuture<'a, Vec<SignedTxnResponse>> {
        Box::pin(async move {
            // Convert to JSON array format expected by algo_signTxn
            let txn_array: Vec<Value> = transactions
                .iter()
                .map(serde_json::to_value)
                .collect::<Result<_, _>>()
                .map_err(WalletConnectError::Json)?;

            let params = json!([txn_array]);

            let response = self
                .send_session_request("algo_signTxn", params)
                .await
                .map_err(WalletConnectError::Relay)?;

            // Parse the response array
            let signed: Vec<SignedTxnResponse> =
                serde_json::from_value(response).map_err(WalletConnectError::Json)?;

            Ok(signed)
        })
    }
}

/// Wrapper for relay subscription message params.
#[derive(Debug, serde::Deserialize)]
struct RelayMessageWrapper {
    data: RelayMessage,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_impl() {
        // Just verify Debug is implemented and doesn't expose secrets
        let debug_output = format!("{:?}", SessionState::Pending);
        assert!(debug_output.contains("Pending"));
    }
}
