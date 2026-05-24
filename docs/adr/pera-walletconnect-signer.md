---
id: pera-walletconnect-signer
title: Pera Wallet signer for the atomic composer
abstract: Integrate the atomic composer with Pera Wallet by adding a PeraSigner that implements the async, group-aware Signer trait in a new optional algonaut_walletconnect crate. It maps SigningRequest to an ARC-0001 algo_signTxn array over an injected, transport-agnostic WalletConnect v2 session, decodes returned blobs through the external-signature-ingress path, and inherits the one-Arc-one-prompt and Send/native-first trade-offs from async-signer-trait. An optional `relay` feature provides a batteries-included WalletConnect v2 relay client.
status: accepted
date: 2026-05-22
deciders: []
tags: [api, async, signing, walletconnect, pera]
---

# Pera Wallet signer for the atomic composer

## Status

Accepted. The concrete follow-up that
[`async-signer-trait`](async-signer-trait.md) explicitly deferred — its
out-of-scope note reads "this ADR does not define a concrete WalletConnect
client." This ADR is that client, for Pera.

Builds on:

- [`async-signer-trait`](async-signer-trait.md) — the async, group-aware
  `Signer` trait this integration plugs into. It is already accepted and
  implemented (`algonaut_transaction::signer`), so this ADR adds no trait
  changes; it consumes the extension point.
- [`external-signature-ingress`](external-signature-ingress.md) — the
  blessed `rmp_serde::from_slice::<SignedTransaction>(&blob)?` path by
  which an out-of-crate signer turns wallet-returned bytes into the
  trait's return value. Pera is precisely the "WalletConnect wallet
  returns signed transaction bytes" case it names.
- [`atomic-transaction-composer-typestate`](atomic-transaction-composer-typestate.md)
  — the `UnsignedAtomicGroup::sign().await` step that drives signers, and
  the `&self` simulate that keeps a dry-run prompt-free.
- [`signer-trait`](signer-trait.md) — the open-trait decision (D7) that
  lets a third-party signer exist at all.

## Context

[`async-signer-trait`](async-signer-trait.md) reshaped `Signer` to be
async and group-aware *for* WalletConnect, then stopped at the SDK
boundary: it defined the extension point and the composer plumbing, but
deliberately left "a concrete WalletConnect client, custodial HTTP
protocol, retry policy, or UI callback model" out of scope. Everything
needed to host a Pera signer is therefore already in place:

```rust
// algonaut_transaction::signer — already shipped.
pub struct SigningRequest<'a> {
    pub transactions: &'a [Transaction], // the full, group-id-stamped group
    pub indexes: &'a [usize],            // the slots this signer must sign
}

pub trait Signer: std::fmt::Debug + Send + Sync {
    fn sign_transactions<'a>(&'a self, request: SigningRequest<'a>) -> SigningFuture<'a>;
}
```

What remains is to write `impl Signer for PeraSigner` and decide where it
lives, how it talks to the wallet, and how its failures surface. That is
this ADR.

### What Pera actually speaks

Pera Wallet is the dominant Algorand mobile wallet. A backend or desktop
dApp reaches it over **WalletConnect v2** (v1 is sunset) and asks it to
sign using the Algorand wallet RPC standardized as **ARC-0001**
(`algo_signTxn`). The wire contract:

- The dApp sends, as the parameters of an `algo_signTxn` JSON-RPC request
  on the established WalletConnect session, an array of `WalletTransaction`
  objects — one per transaction in the group:

  ```jsonc
  // algo_signTxn params: [ WalletTransaction[] ]
  [
    { "txn": "<base64 msgpack of unsigned txn 0>" },              // sign this
    { "txn": "<base64 msgpack of unsigned txn 1>", "signers": [] } // display only
  ]
  ```

  `txn` is canonical msgpack of the *unsigned* transaction, base64-encoded.
  `signers: []` means "this slot is part of the group for context — show it
  to the user but do **not** sign it." Omitting `signers` (or listing the
  connected address) means "sign this slot."

- The wallet returns an array aligned to the request: a base64 signed-txn
  blob for each slot it signed, and `null` for each `signers: []` slot.

### The request shape is already the right shape

This is not a coincidence: `async-signer-trait` designed `SigningRequest`
around exactly this wire format. The mapping is mechanical —
`request.transactions` is the `WalletTransaction[]` array; `request.indexes`
selects which entries get a non-empty/absent `signers` and which get
`signers: []`. The "wallet sees the whole atomic group, signs only its
slots, in one approval round-trip" promise *is* ARC-0001's `signers`
field. So the integration is mostly a faithful codec plus the
already-blessed blob decode — not new protocol design.

### The forces that shape the decision

- **WalletConnect v2 is a heavy, evolving, runtime-specific dependency.**
  A real session needs a relay websocket client, pairing and session
  cryptography, and JSON-RPC framing. That surface does not belong in a
  leaf crate like `algonaut_transaction`, which must stay buildable for
  offline signing and for wasm consumers that never touch a relay.
- **The `Send` bound bites here specifically.**
  [`async-signer-trait`](async-signer-trait.md) took `SigningFuture: Send`
  for the first cut and deferred a `?Send` variant. Pera-in-the-browser
  (via Pera Connect's JS interop) is the canonical `!Send` case that
  deferral named. We have to say which side of that line this ADR lands
  on.
- **The trait exposes no address, on purpose.** Neither the composer nor
  the trait knows which slots are "the Pera account's" — that was the
  D6/D7 multisig/HSM rationale, and it is why the composer groups by
  `Arc::ptr_eq`. But ARC-0001's `signers` field and Pera's own
  sender-vs-session check both need the connected address. So the address
  has to live *inside* the signer, supplied at connect time, not on the
  trait.
- **There is no typed error channel for remote signing.**
  `TransactionError` (the trait's error type) has no opaque variant; its
  closest catch-all is `Deserialization(String)`. User rejection, an
  expired session, a relay timeout, and a sender mismatch have nowhere
  structured to go. `async-signer-trait` flagged this as needed follow-up;
  Pera is the integration that forces it.

## Decision

Add a `PeraSigner` that implements the existing async `Signer` trait, in a
**new optional crate**, mapping `SigningRequest` to an ARC-0001
`algo_signTxn` exchange over an **injected, transport-agnostic**
WalletConnect session, and decoding the response through the
external-signature-ingress path. No change to the `Signer` trait or the
composer.

### A new optional crate, `algonaut_walletconnect`

The signer lives in a new workspace member, `algonaut_walletconnect`, not
in `algonaut_transaction`:

```
algonaut_walletconnect
├── depends on algonaut_transaction  (Signer, SigningRequest, SignedTransaction, Transaction)
└── depends on algonaut_core         (Address)
```

`algonaut_transaction` does **not** depend on it — the dependency points
one way, so the heavy WalletConnect/relay/JSON-RPC surface never reaches
the leaf crate or its wasm/offline consumers. The umbrella `algonaut`
crate re-exports it behind an off-by-default `walletconnect` feature.

`PeraSigner` is a thin preset over a generic `WalletConnectSigner`: Pera
is "WalletConnect v2 + ARC-0001 with Pera's chain/metadata defaults."
Lute, Defly, and other ARC-0001 wallets differ only in connection
metadata, so the generic carries the logic and `PeraSigner` is the
ergonomic entry point.

### `PeraSigner` implements the async trait directly

```rust
#[derive(Debug, Clone)]
pub struct PeraSigner {
    /// The account this session is connected to. Supplied at connect
    /// time, because the trait deliberately exposes no address.
    address: Address,
    /// The established WalletConnect session — injected, see below.
    session: Arc<dyn WalletConnectSession>,
}

impl Signer for PeraSigner {
    fn sign_transactions<'a>(&'a self, request: SigningRequest<'a>) -> SigningFuture<'a> {
        Box::pin(async move {
            let params = self.encode_arc0001(&request)?;        // codec, below
            let blobs  = self.session.algo_sign_txn(params).await?;
            self.decode_response(&request, blobs)               // ingress, below
        })
    }
}
```

The signer owns three responsibilities: the ARC-0001 codec, driving the
session, and validating the response. It does **not** own the relay.

### The transport is injected, not embedded

`PeraSigner` talks to the wallet through a narrow async trait, supplied by
the caller (or a companion transport crate):

```rust
/// An established WalletConnect v2 session against an ARC-0001 wallet.
/// The dApp creates and pairs it; the signer only issues requests on it.
pub trait WalletConnectSession: std::fmt::Debug + Send + Sync {
    /// Issue an `algo_signTxn` request and await the wallet's response.
    /// Returns one entry per requested transaction, aligned to the
    /// `WalletTransaction[]` order: `Some(blob)` for a signed slot,
    /// `None` for a `signers: []` (display-only) slot.
    fn algo_sign_txn<'a>(
        &'a self,
        params: Vec<WalletTransaction>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Option<Vec<u8>>>, PeraError>> + Send + 'a>>;
}
```

algonaut owns the part it can own well and test offline — the ARC-0001
encoding, the response decode, and the validation — and leaves the relay
websocket, pairing, and session lifecycle to the transport implementation.
This mirrors [`external-signature-ingress`](external-signature-ingress.md)
keeping the trait bytes-in/bytes-out: the protocol mapping is small,
stable, and unit-testable against a mock session; the relay client is
heavy, fast-moving, and native-vs-wasm-specific. Pushing the relay behind
a trait keeps that split out of the signer, and lets the integration be
exercised in CI with a fake `WalletConnectSession` that returns canned
blobs — no live wallet, no relay.

Connection establishment (QR/deep-link pairing, session approval, the
`Address` handshake) is the transport's concern and out of scope here; the
signer receives an already-connected session and a known address.

### Request codec: `SigningRequest` → `WalletTransaction[]`

```rust
fn encode_arc0001(&self, req: &SigningRequest) -> Result<Vec<WalletTransaction>, PeraError> {
    req.transactions.iter().enumerate().map(|(i, txn)| {
        let bytes = txn.to_msg_pack()?;                 // canonical unsigned msgpack
        Ok(if req.indexes.contains(&i) {
            // A slot we own: ask the wallet to sign it as the connected account.
            WalletTransaction { txn: bytes, signers: None /* => sign */ }
        } else {
            // Context only: show it in the group, do not sign it.
            WalletTransaction { txn: bytes, signers: Some(vec![]) }
        })
    }).collect()
}
```

Every group transaction becomes a `WalletTransaction`, so the wallet
renders the whole atomic group; only `request.indexes` carry a signable
`signers`. That is the group-aware contract realized over the wire: one
`algo_signTxn`, the user sees the full group, signs exactly our slots.

### Response decode through the blessed ingress

```rust
fn decode_response(
    &self,
    req: &SigningRequest,
    blobs: Vec<Option<Vec<u8>>>,
) -> Result<Vec<SignedTransaction>, PeraError> {
    // One signed blob per requested index, in request order.
    req.indexes.iter().map(|&i| {
        let blob = blobs.get(i).and_then(Option::as_ref)
            .ok_or(PeraError::MissingSignature { index: i })?;
        // The single public ingress (external-signature-ingress):
        let signed: SignedTransaction = rmp_serde::from_slice(blob)?;
        Ok(signed)
    }).collect()
}
```

The decode is the one-liner that
[`external-signature-ingress`](external-signature-ingress.md) blessed —
`transaction_id` is recomputed from the decoded transaction, so a wallet
that returns a blob with a stale id cannot violate the
`SignedTransaction` invariant. The composer's own step-4 validation
(count, order, and id-must-match-the-requested-transaction, in
`sign_group`) is the backstop against a wallet returning a signature for
the *wrong* transaction. The signer adds one Pera-specific **pre-flight**
check before sending: every requested slot's sender (or rekey `auth_addr`)
must equal the connected `address`, failing fast with
`PeraError::SenderMismatch` rather than waiting for the wallet to reject a
malformed request.

### Identity and the one-prompt contract, concretely

[`async-signer-trait`](async-signer-trait.md) grouped signing by
`Arc::ptr_eq` and made "one wallet, one prompt" contingent on the caller
sharing a single `Arc<dyn Signer>` clone across the wallet's slots. For
Pera that contract has an exact reading:

> **one `Arc<PeraSigner>` == one session == one connected account == one
> `algo_signTxn` round-trip == one approval.**

```rust
let pera: Arc<dyn Signer> = Arc::new(PeraSigner::new(addr, session));
let outcome = AtomicGroupBuilder::new()
    .add_method_call(MethodCall::builder(app_id, method)
        .sender(addr).signer(pera.clone()).build()?)
    .add_transaction(TransactionWithSigner::new(payment, pera.clone())) // same Arc
    .build()?
    .sign().await?      // one Pera prompt for both slots
    .execute(&algod).await?;
```

Cloning the `Arc` keeps it one prompt; constructing a *second*
`PeraSigner` for the same wallet would group as two signers and prompt
twice. A genuinely different Pera account is a different session and a
different `PeraSigner` — and a second prompt there is correct, because it
is a second account's separate approval.

### `Send` and native-first; browser deferred

`WalletConnectSession`'s future is `+ Send`, so `PeraSigner`'s
`SigningFuture` is `Send` and `UnsignedAtomicGroup::sign()` stays `Send` on
the multi-threaded native executor — the primary target (backend services,
CLIs, desktop dApps using a native WC v2 relay client). The browser case —
Pera Connect's JS interop, which yields `!Send` futures — is **explicitly
deferred** to the `?Send` variant that
[`async-signer-trait`](async-signer-trait.md) already earmarked, rather
than built speculatively now. In practice a browser dApp typically drives
Pera Connect (JS) directly anyway; the native relay path is where a Rust
SDK adds the most. We inherit that ADR's trade, we do not re-litigate it.

### Errors: an opaque external-signer channel

The Pera crate gets a typed `PeraError` (a structured leaf-crate error per
[`structured-leaf-errors`](structured-leaf-errors.md)):

```rust
pub enum PeraError {
    UserRejected,                          // wallet user declined
    SessionExpired,                        // / disconnected
    RelayTimeout { timeout: Duration },
    SenderMismatch { index: usize },       // slot not owned by the connected account
    ChainMismatch,                         // session on a different network
    MissingSignature { index: usize },     // wallet returned null for a requested slot
    MalformedResponse(String),
    BlobDecode(rmp_serde::decode::Error),  // from the ingress path
}
```

For these to surface from `Signer::sign_transactions` (which returns
`TransactionError`), `algonaut_transaction` needs an **opaque external
variant** so it can carry a third-party signer's error without depending
on the Pera crate:

```rust
// algonaut_transaction::error::TransactionError — added by this ADR.
#[error("signer error: {0}")]
Signer(#[source] Box<dyn std::error::Error + Send + Sync>),
```

`impl Signer for PeraSigner` maps `PeraError` into
`TransactionError::Signer(Box::new(e))`. This discharges the "async signer
errors need structured follow-up" item from
[`async-signer-trait`](async-signer-trait.md) — the dependency stays
one-directional and the typed detail stays inspectable via
`source()`/downcast.

### Simulate stays prompt-free, for free

Because [`async-signer-trait`](async-signer-trait.md) made the simulate
path placeholder-sign every slot and never call
`Signer::sign_transactions`, `unsigned.simulate(&algod).await?` on a group
carrying a `PeraSigner` never reaches the wallet — no approval pops for a
dry-run. The one-group `simulate → sign → execute` flow holds with a Pera
signer attached, and this ADR adds nothing to make it so; it just relies
on the property already decided. Only `sign()` (and the `execute` after
it) issues `algo_signTxn`.

### Optional relay feature: batteries-included transport

While the `WalletConnectSession` trait keeps the transport injectable and
testable, requiring users to implement it themselves creates friction. To
provide a turnkey solution, `algonaut_walletconnect` includes an **optional
`relay` feature** that provides a concrete WalletConnect v2 relay
implementation:

```toml
[features]
default = []
relay = ["tokio-tungstenite", "x25519-dalek", "chacha20poly1305", "hkdf", "sha2"]
```

When enabled, the crate exports:

```rust
/// A WalletConnect v2 relay client that implements WalletConnectSession.
pub struct WalletConnectRelay {
    // WebSocket connection to wss://relay.walletconnect.com
    // X25519 key exchange for session encryption
    // ChaCha20-Poly1305 symmetric encryption
    // JSON-RPC message framing
    // Pairing/session state machine
}

impl WalletConnectRelay {
    /// Create a new relay client with a WalletConnect project ID.
    pub async fn new(project_id: &str) -> Result<Self, RelayError>;

    /// Generate a pairing URI for QR code / deep link.
    pub fn pairing_uri(&self) -> String;

    /// Wait for a wallet to pair and return the connected address.
    pub async fn wait_for_session(&self) -> Result<Address, RelayError>;
}

impl WalletConnectSession for WalletConnectRelay { /* ... */ }
```

This design preserves the separation of concerns:

- **Without `relay`:** Lean codec-only crate (~50 KB), users inject their
  own transport, CI tests run against mock sessions.
- **With `relay`:** Full batteries-included solution (~500 KB with crypto
  deps), ready to connect to Pera out of the box.

The relay implementation handles:

- WebSocket connection to `wss://relay.walletconnect.com`
- WalletConnect v2 pairing protocol (symmetric key derivation)
- Session proposal/approval handshake
- ChaCha20-Poly1305 envelope encryption
- JSON-RPC 2.0 request/response framing
- Algorand chain ID configuration (416001 MainNet, 416002 TestNet)

## What this ADR explicitly does not propose

- **The `?Send` / wasm-browser variant.** Inherited deferral from
  `async-signer-trait`; built when a real browser Pera integration lands.
  The `relay` feature targets native (tokio) runtimes only.
- **WalletConnect v1.** Sunset; not targeted.
- **RPC methods beyond `algo_signTxn`.** `algo_signData`, multi-account
  selection within one session, and ARC-0025 (`algorand://`) URIs are out
  of scope. One signer is bound to one connected address.
- **Signature verification.** The decode path trusts the bytes; the
  composer validates count/order/id, and cryptographic validity is
  algod's call at submit — as `external-signature-ingress` already
  decided.

## Consequences

- **Pera becomes a first-class atomic-composer signer with zero trait
  churn.** `async-signer-trait` and `external-signature-ingress` did the
  enabling work; this ADR is `impl Signer for PeraSigner` plus a codec,
  not a re-shaping of the signing model.
- **The heavy WalletConnect surface stays quarantined.** It lives in an
  optional `algonaut_walletconnect` crate behind a feature flag;
  `algonaut_transaction` stays leaf, offline-buildable, and free of relay
  dependencies.
- **The protocol mapping is algonaut-owned and testable without a wallet.**
  The ARC-0001 codec, the blob decode, and the validation are unit-testable
  against a mock `WalletConnectSession`; the relay is out of the test path.
- **Transport injection keeps the native/wasm split out of the signer.**
  The `Send`/`!Send` decision lives in the transport impl, not in
  `PeraSigner`, so the signer doesn't have to fork.
- **The one-`Arc`-one-prompt contract gains a concrete instance and a
  concrete footgun.** Sharing the `Arc<PeraSigner>` clone yields a single
  approval; a duplicate `PeraSigner` for the same wallet double-prompts.
  The SDK can't enforce it, so it is documented at the `PeraSigner`
  constructor.
- **`Send`/native-first is inherited; the browser path is a named
  follow-up,** not an omission.
- **One new `TransactionError` variant.** `TransactionError::Signer(Box<dyn
  Error + Send + Sync>)` is additive (pre-1.0; a new enum variant), and it
  discharges `async-signer-trait`'s deferred async-signer-error item for
  every remote signer, not just Pera.
- **Simulate is prompt-free at no extra cost,** by relying on the
  placeholder-only simulate path already decided.
- **Risk surface:** correctness now depends on (a) the injected
  transport faithfully implementing `algo_signTxn`, and (b) the maturity
  of the Rust WalletConnect v2 ecosystem the eventual default transport
  will sit on. Both are isolated behind the `WalletConnectSession` seam,
  so they can evolve without touching the signer or the composer.
- **The `relay` feature provides a turnkey solution.** Users who enable
  `walletconnect` with `relay` get a working Pera integration out of the
  box — no need to implement `WalletConnectSession` themselves. The
  trade-off is heavier dependencies (tokio-tungstenite, crypto crates),
  which is why it's opt-in.
