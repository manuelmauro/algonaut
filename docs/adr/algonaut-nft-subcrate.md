---
id: algonaut-nft-subcrate
title: An algonaut_nft subcrate for all NFT use cases
abstract: 'Introduce algonaut_nft, an offline-first workspace crate covering every NFT-related ARC: ASA NFT minting with ARC-3/ARC-69 metadata, ARC-19 reserve-address mutability, ARC-16 traits / ARC-36 filters / ARC-53 collections, ARC-71 soulbound lifecycle, ARC-72 smart-contract NFTs, ARC-18 royalty enforcement, and an ARC-74 indexer client. The offline core (typed metadata models, hashing/integrity, ABI call builders) is always on; network helpers ride additive algod/indexer/fetch features; it is re-exported as algonaut::nft behind a default-off nft feature. ARC-49 (a governance program) is out of scope.'
status: accepted
date: 2026-06-19
deciders: []
tags: [nft, arc, crate-layout, features, metadata, asa, wasm]
---

# An algonaut_nft subcrate for all NFT use cases

## Status

Accepted. Being implemented on the `feat/algonaut-nft` branch: new workspace
member `algonaut_nft`, re-exported as `algonaut::nft` behind a default-off `nft`
feature. The first cut lands the full offline core (metadata models, ARC-19 URL
transforms, integrity hashing, ASA presets + ARC-71 lifecycle, ARC-72/ARC-18
ABI bindings, ARC-74 types) plus an **ARC-89 preview** (the Asset Metadata Box
codec, flags, box-name, and registry constants — see D10). Network helpers
(HTTP/IPFS fetch, ARC-74 client) ride an opt-in `fetch` feature. As decided with
the branch owner, **no backward-compatibility guarantees** apply to this crate
while it stabilises, and the ARC-89 surface is explicitly marked unstable/preview.

## Context

"NFT support" on Algorand is not one feature; it is roughly a dozen ARCs that
fall into two families with almost nothing in common at the protocol level —
NFTs that are **Algorand Standard Assets** (ASAs) and NFTs that are **smart
contracts**. A survey of every ARC that mentions NFTs:

| ARC | Status | Family | Concern |
|-----|--------|--------|---------|
| ARC-3 | Final | ASA | Off-chain JSON metadata; pure/fractional NFT shape (`t`/`dc`); `un`/`an`/`au`/`am` conventions; SRI `*_integrity` |
| ARC-69 | Final (→ ARC-89) | ASA | On-chain metadata in the latest `acfg` `note`; `media_url`/`mime_type`/`#x` URL fragment; retrofittable |
| ARC-19 | Final (→ ARC-89) | ASA | Mutable pointer via `template-ipfs://{ipfscid:…:reserve:…}` reusing the Reserve address as a CID bitbucket |
| ARC-16 | Final | metadata | `traits` object for rarity |
| ARC-36 | Final | metadata | `filters` object for non-rarity filtering |
| ARC-53 | Last Call | metadata | Collection-level declaration (banners, royalties, scoping across wallets) |
| ARC-71 | Final | ASA | Non-transferable / "soulbound" ASA via freeze + zeroed clawback, with a revoke (not burn) lifecycle |
| ARC-18 | Final | app | Enforced royalties on every transfer via an ARC-20 clawback-routed app |
| ARC-72 | Living | app | ERC-721-style smart-contract NFT ABI (`arc72_ownerOf`, `arc72_transferFrom`, …) + extensions |
| ARC-74 | Final | app | REST indexer API for ARC-72 tokens (`/nft-indexer/v1/tokens`, `/transfers`) |
| ARC-49 | Deprecated | — | A marketplace ALGO-rewards governance program — **no code** |

Today the repo has the raw protocol layer for the ASA family but nothing
NFT-aware on top of it:

- `algonaut_transaction::builder` already has `CreateAsset`, `ConfigureAsset`,
  `TransferAsset`, `AcceptAsset`, `FreezeAsset`, `ClawbackAsset`, `DestroyAsset`
  (one builder per transaction, per
  [`one-build-per-transaction`](one-build-per-transaction.md)), and
  `algonaut_model` carries the `AssetParams` (`apar`) shape for both algod and
  indexer.
- `algonaut_core` has the `AssetId` / `AppId` / `Address` newtypes
  ([`identifier-newtypes-at-client-boundary`](identifier-newtypes-at-client-boundary.md)).
- `algonaut_abi` + the `contract!` macro
  ([`contract-macro-arc56-app-spec`](contract-macro-arc56-app-spec.md)) can
  call arbitrary ARC-4/ARC-56 contracts, and `algonaut::atomic` composes and
  signs groups.
- `algonaut_crypto` has the SHA-512/256 primitive the metadata hashes need.

So none of the NFT ARCs require new protocol code; they all need **conventions
layered on top** — metadata models, the ARC-19 CID↔Reserve transform, integrity
hashing, NFT-shaped ASA-config presets, the ARC-71 lifecycle, ARC-72/ARC-18 ABI
bindings, and the ARC-74 client. Right now a user must hand-assemble all of it
from the raw builders and a pile of `serde_json`, re-deriving the same `am`
hashing and `template-ipfs` parsing everyone else does. That is exactly the
"convention nobody should re-implement" shape the SDK exists to absorb.

Three prior decisions constrain *where* this code can live:

1. **Offline crates stay offline.**
   [`client-feature-gates`](client-feature-gates.md) made the offline
   foundation (`core`/`crypto`/`encoding`/`abi`/`transaction`/`model`)
   unconditional and pushed every `reqwest`-touching module behind `algod` /
   `indexer` / `kmd` features, explicitly so `wasm32` "build a transaction in
   the browser, hand it to a wallet to sign" consumers pay for no HTTP/TLS
   stack. Most of the NFT surface (metadata models, hashing, CID math, ABI
   call construction) is pure offline computation and must not drag in a client.
2. **Shared, dependency-light concerns become their own leaf crate.**
   [`abi-json-model-shared-leaf-crate`](abi-json-model-shared-leaf-crate.md)
   extracted `algonaut_abi_model` rather than bloating an existing crate or
   duplicating a wire format, accepting "one more published crate" as the price
   — and [`publish-entire-workspace`](publish-entire-workspace.md) commits to
   publishing every member, with features (not crate exclusion) gating optional
   weight.
3. **Naming omits redundant nouns.**
   [`transaction-naming-convention`](transaction-naming-convention.md): a
   module/type should not repeat a noun its crate, parent module, or argument
   already carries.

The open question this ADR answers: do we add a high-level `nft` *module* to
the umbrella crate (like `atomic`/`simulate`/`dryrun`), or a new `algonaut_nft`
*member crate*? And how is its large, two-family surface organised and gated so
the offline parts stay offline?

## Decision

Add **`algonaut_nft`**, a new workspace member that owns the entire NFT
convention layer, re-exported as `algonaut::nft`. It is **offline-first**: its
default surface depends only on the offline foundation crates and links no HTTP
client; network helpers are opt-in features.

### D1 — A member crate, not an umbrella module

`atomic`/`simulate`/`dryrun` live *in* the umbrella crate because they are
hard-coupled to `algonaut_algod::models` and cannot compile without the algod
client ([`client-feature-gates`](client-feature-gates.md), D2). The NFT
convention layer is the opposite: its core is pure offline computation over
types that already exist in `algonaut_core` / `algonaut_transaction` /
`algonaut_abi`. A standalone crate therefore:

- lets a `wasm32` or offline consumer depend on `algonaut_nft` directly for
  metadata + minting params + ABI call building with **no** `reqwest` in the
  graph, the headline win of [`client-feature-gates`](client-feature-gates.md);
- keeps one cohesive home for ~a dozen ARCs instead of swelling
  `algonaut`'s `lib.rs`;
- is publishable on its own cadence under
  [`publish-entire-workspace`](publish-entire-workspace.md).

The umbrella re-exports it behind an **additive, default-off** `nft` feature:

```rust
#[cfg(feature = "nft")]
pub use algonaut_nft as nft;
```

Default-off is non-breaking — the crate is new, nobody depends on it, and the
default build stays lean — while `algonaut = { version = "…", features = ["nft"] }`
turns it on. This mirrors the additive philosophy of
[`client-feature-gates`](client-feature-gates.md) D1 without changing any
existing default surface.

### D2 — Dependency layering: offline core, gated edges

`algonaut_nft`'s mandatory dependencies are offline only:
`algonaut_core`, `algonaut_crypto`, `algonaut_encoding`, `algonaut_abi`,
`algonaut_transaction`, plus `serde` / `serde_json` and a small `data-encoding`
/ multibase/multihash surface for CIDs. It pulls **no** client crate by default.

The network edges become the crate's own features, which the umbrella's `nft`
feature can forward:

```toml
# algonaut_nft/Cargo.toml
[features]
default = []
# ARC-74 NFT indexer client.
indexer = ["dep:algonaut_indexer"]
# Live reads from algod: resolve an asset's params, current ARC-69 note,
# ARC-72 on-chain state via read-only ABI calls.
algod   = ["dep:algonaut_algod"]
# Convenience HTTP(S)/IPFS-gateway fetch of off-chain metadata (otherwise the
# caller fetches the resolved URL themselves — see D6).
fetch   = ["dep:reqwest"]
```

The default (`[]`) is the offline core. This is the same discipline
[`client-feature-gates`](client-feature-gates.md) applies one level up, applied
inside the crate so its own offline/online split is explicit and `cargo hack
--feature-powerset` can police it.

### D3 — Module shape, organised by concern, ARC-faithful underneath

```
algonaut_nft
├── lib.rs            // NftError (D7), prelude, re-exports
├── metadata/         // the off/on-chain JSON + note conventions
│   ├── arc3.rs       //   arc3::Metadata, Localization, pure/fractional shape checks
│   ├── arc69.rs      //   arc69::Metadata (note body), MediaType (#i/#v/#a/#p/#h)
│   ├── traits.rs     //   ARC-16 `traits`
│   ├── filters.rs    //   ARC-36 `filters`
│   ├── collection.rs //   ARC-53 declaration
│   └── integrity.rs  //   am hashing + SRI *_integrity compute/verify (D5)
├── url.rs            // ARC-19 template-ipfs parsing + CID<->Reserve (D4)
├── asa.rs            // NFT-shaped ASA presets + ARC-71 soulbound lifecycle (D8)
├── arc72.rs          // smart-contract NFT ABI bindings + ARC-73 detection (D9)
├── royalty.rs        // ARC-18 enforcer client (D9)
└── indexer.rs        // ARC-74 client  [feature = "indexer"]
```

Naming follows
[`transaction-naming-convention`](transaction-naming-convention.md): the type
in `arc3.rs` is `arc3::Metadata`, **not** `Arc3Metadata` (the module already
says ARC-3), and lives under `metadata::arc3` so `nft::metadata::arc3::Metadata`
reads without stutter. Where a single ARC *is* the standard (19/71/72/18/74) the
module is concern-named (`url`, `asa`, `arc72`, `royalty`, `indexer`) and the
ARC is documented at the top; where several ARCs compose one artifact (the
metadata JSON: 3 + 16 + 36) they share the `metadata` module.

The serde models are kept **faithful to each spec** — distinct `arc3::Metadata`
and `arc69::Metadata` rather than one merged struct — because the two are stored
differently (off-chain file vs `acfg` note), hash differently, and evolve
independently; merging them would force lossy round-trips and violate the
vendor-fidelity habit the repo already follows for spec fixtures. A higher-level
read path (D6) normalises across them for callers who just want "the image and
the traits."

### D4 — ARC-19 reserve-address mutability as a typed transform

`url.rs` provides total, offline conversions between a `template-ipfs://…`
asset URL and an IPFS CID carried in the 32-byte Reserve `Address`:

```rust
pub struct TemplateIpfsUrl { /* version, codec, field=reserve, hash */ }
pub fn parse_template_url(au: &str) -> Result<TemplateIpfsUrl, NftError>;
pub fn cid_from_reserve(t: &TemplateIpfsUrl, reserve: Address) -> Result<Cid, NftError>;
pub fn reserve_from_cid(cid: &Cid) -> Result<Address, NftError>;   // for minting
pub fn resolve_url(au: &str, reserve: Address) -> Result<String, NftError>; // -> ipfs://…
```

Clients MUST support CID v0/v1, `raw` + `dag-pb`, and `sha2-256` per the ARC.
The "mutate" operation is just `reserve_from_cid` feeding the existing
`ConfigureAsset` builder's `.reserve(..)` — `algonaut_nft` adds the address math,
not a new transaction type. Per ARC-19, combined ARC-3+ARC-19 assets skip `am`
validation; D5's verifier exposes that as an explicit opt-out rather than a
silent pass.

### D5 — Integrity and hashing live in one place

`metadata::integrity` owns the two hash schemes so no caller re-derives them:

- **ARC-3 `am`** — SHA-256 of the JSON when there is no `extra_metadata`, else
  the domain-separated `SHA-512/256("arc0003/am" || SHA-512/256("arc0003/amj" ||
  json) || e)` form. `compute_metadata_hash(&Metadata) -> [u8; 32]` and a
  `verify` counterpart, built on `algonaut_crypto`.
- **SRI `*_integrity`** — the W3C subresource-integrity strings ARC-3 attaches
  to each URI (`image_integrity`, `animation_url_integrity`, …). Compute/verify
  over caller-supplied bytes; the crate never fetches to verify (that is the
  caller's or the `fetch` feature's job, D6).

### D6 — Transport-agnostic resolution by default

Resolving an NFT means: read its `AssetParams`, pick the metadata source (ARC-3
URL, ARC-69 note, or ARC-19 template), fetch bytes, parse, verify integrity. The
crate makes **every step that is not I/O** available offline and leaves the I/O
to the caller:

- offline: given `AssetParams` (+ the latest `acfg` note for ARC-69), return the
  resolved metadata URL and a parsed model from caller-provided bytes;
- `feature = "algod"`: a thin resolver that reads params/notes via an injected
  `algonaut::Algod` and returns the same;
- `feature = "fetch"`: a convenience that actually pulls `https`/`ipfs`-gateway
  bytes (honouring the ARC-3 CORS note) and verifies integrity end-to-end.

Keeping the default transport-agnostic is what preserves the wasm/offline
promise of D2: a browser consumer resolves metadata with its own `fetch()`,
linking no `reqwest`.

### D7 — One structured error type

A `thiserror` `NftError` with typed variants (`BadTemplateUrl`,
`UnsupportedCid`, `MetadataHashMismatch`, `IntegrityMismatch`, `NotPureNft`,
`InvalidArc69Note`, `InterfaceUnsupported`, gated `Indexer`/`Algod`/`Fetch`
wrappers), per
[`structured-leaf-errors`](structured-leaf-errors.md) — no `Msg(String)`
catch-all. Network-feature variants are `#[cfg]`-gated so the offline build's
error enum carries no client types.

### D8 — ASA NFTs: presets and the ARC-71 lifecycle, not new transactions

`asa.rs` wraps the existing builders rather than replacing them:

- `mint_pure(..)` / `mint_fractional(power)` set `total`/`decimals` to the
  ARC-3 pure (`1`/`0`) and fractional (`10^n`/`n`) shapes and attach an
  `arc3::Metadata` (URL + `am`) or `arc69::Metadata` (note) — returning a
  configured `CreateAsset` the caller still builds, signs, and submits through
  the normal pipeline. No new transaction type, no signing inside the crate.
- ARC-71 soulbound is modelled as a small typestate over the three spec
  states — **Issued** (clawback `ZeroAddress`, freeze = issuer), **Held**
  (freeze `ZeroAddress`, holder account frozen), **Revoked** (manager
  `ZeroAddress`) — each transition emitting the correct `CreateAsset` /
  `FreezeAsset` / `ConfigureAsset`. The crate encodes the invariants (decimals
  0, total 1, the address-field rules) so an illegal lifecycle move is a type
  error, echoing the typestate spirit of
  [`atomic-transaction-composer-typestate`](atomic-transaction-composer-typestate.md).

### D9 — Smart-contract NFTs (ARC-72) and royalties (ARC-18) as ABI bindings

These are *application* standards, so they reuse the ABI stack, not new
plumbing:

- `arc72.rs` exposes the canonical method set (`arc72_ownerOf`,
  `arc72_transferFrom`, `arc72_approve`, `arc72_setApprovalForAll`,
  `arc72_balanceOf`, `arc72_totalSupply`, `arc72_tokenByIndex`,
  `arc72_tokenURI`, …) as `MethodCall` builders over
  [`method-call-builder`](method-call-builder.md), plus ARC-73 interface
  detection by selector (core `0x53f02a40`, metadata `0xc3c1fc00`, …). Bindings
  are hand-pinned to the spec signatures — like
  [`kmd-client-hand-written`](kmd-client-hand-written.md), the surface is small,
  stable, and standardised, so hand-written beats codegen, and unlike ARC-18
  there is no single app to point `contract!` at (ARC-72 is an interface across
  many contracts). Read-only methods are composed via the existing simulate
  path. The crate builds calls; execution stays with `algonaut::atomic`.
- `royalty.rs` covers ARC-18's enforcer interface (`set_policy`,
  `set_payment_asset`, `offer`, `transfer_algo_payment`,
  `transfer_asset_payment`, `royalty_free_move`, read-only `get_policy` /
  `get_offer`) and the `RoyaltyPolicy` / `AssetOffer` types. Because ARC-18 ships
  a concrete ARC-4 app spec, this binding MAY instead be generated by the
  existing `contract!` macro
  ([`contract-macro-arc56-app-spec`](contract-macro-arc56-app-spec.md)) from the
  vendored spec; the ADR leaves the generate-vs-handwrite choice to
  implementation, but the *types and module* are fixed here.

### D10 — Scope boundaries

- **ARC-49 is out of scope** — it is a deprecated Foundation rewards *program*
  with no on-chain format or ABI to model.
- **ARC-89** (Last Call; supersedes the ARC-19/69 line, extends ARC-3) ships in
  this cut as an explicitly-**preview** `arc89` module: an offline, byte-exact
  codec for the Asset Metadata Box (51-byte header — identifiers/reversible/
  irreversible flag bytes, 32-byte hash, last-modified round, deprecated-by — plus
  the JSON body), the domain-separated (`arc0089/header|page|am`) metadata-hash
  computation, the 8-byte big-endian box name, the `algorand://…?box=` URL
  convention, and the trusted-registry app-id constants. The registry's ABI is
  documented as signature constants but the on-chain client is deferred. The
  module is marked unstable and exempt from the crate's (already absent) compat
  guarantees until ARC-89 reaches Final.
- **No pinning/uploading.** The crate computes CIDs and reserve addresses and
  verifies integrity; it does not run an IPFS node or upload media. That is a
  deployment concern left to the caller (transport-agnostic, per D6).
- **No marketplace logic.** ARC-53 collections and ARC-18 policies are modelled
  and (de)serialised; matching engines, listings, and order books are not.

## Consequences

- **One coherent home for a sprawling surface.** A user gets ASA NFTs,
  on/off-chain metadata, mutable URLs, traits/filters/collections, soulbound
  tokens, smart-contract NFTs, royalties, and indexing from a single crate that
  reuses the existing builders, newtypes, ABI stack, and atomic composer instead
  of duplicating them.
- **The offline/wasm promise is preserved.** Default `algonaut_nft` links no
  HTTP client; a browser or signing-only consumer gets metadata + minting params
  + ABI call construction with no `reqwest`, exactly the capability
  [`client-feature-gates`](client-feature-gates.md) created. The cost is real
  `#[cfg]` discipline across D2/D6/D7 and a feature-powerset CI job, the same
  tax that ADR already accepted.
- **One more published crate.** Per
  [`publish-entire-workspace`](publish-entire-workspace.md) `algonaut_nft` joins
  the workspace `members` and gets published; the umbrella gains a default-off
  `nft` feature and a `dep:algonaut_nft`. Non-breaking: no existing build
  changes unless it opts in.
- **Spec fidelity over convenience.** Keeping `arc3::Metadata` and
  `arc69::Metadata` distinct (D3) means two models to learn and a normalisation
  layer (D6) to bridge them, but it keeps each byte-faithful to its spec and its
  hashing rules — the safer default for an interop SDK, and consistent with how
  the repo treats vendored spec artifacts.
- **ARC-72/ARC-18 ride the ABI stack.** No new execution machinery; bindings are
  `MethodCall`s run through the atomic composer. This keeps the crate thin but
  couples it to the ABI/contract decisions — a deliberate reuse, and the reason
  `arc72`/`royalty` need no `algod` dependency to *build* calls (only to *read*
  on-chain state, which rides the `algod` feature).
- **Sequencing.** The crate can land in slices that each compile and test
  standalone: (1) `metadata` + `url` + `integrity` + `NftError` (pure offline,
  the bulk of the value); (2) `asa` presets + ARC-71 lifecycle; (3) `arc72` +
  `royalty` ABI bindings; (4) `indexer` (ARC-74) behind its feature; (5) the
  `algod`/`fetch` resolvers. Slice 1 is shippable without any of the others.
- **Known follow-ups.** ARC-89 convergence (D10); a possible `contract!`-generated
  ARC-18 client (D9); whether ARC-74's spec-internal endpoint inconsistency
  (`/nft-index/` vs `/nft-indexer/`) needs a configurable base path — flagged for
  the indexer slice.
- **Status is Proposed.** Accepting this commits the crate's name, layering,
  module boundaries, and feature axes; the per-ARC type details (D3–D9) are
  specified closely enough to start slice 1 but may be refined as each slice
  meets the actual specs and tests.
