---
id: domain-types-serialize-for-both-json-and-msgpack
title: Domain types serialize for both JSON and msgpack
abstract: Branch Address, MultisigSignature, VotePk, VrfPk, StateProofPk, HashDigest and the algonaut_encoding byte helpers on serializer.is_human_readable() so JSON renders canonical strings and msgpack keeps the existing raw-byte wire format.
status: accepted
date: 2026-05-18
deciders: []
tags: []
---

# Domain types serialize for both JSON and msgpack

## Status

Accepted

## Context

Algorand's REST APIs use two wire formats with different conventions:

- **JSON** (the indexer, swagger, dryrun-as-JSON, simulate-as-JSON):
  addresses are base32 checksum strings; byte slices are base64 strings.
- **Msgpack** (transaction signing, block payloads, `Content-Type:
  application/x-binary` endpoints): addresses are raw 32-byte arrays;
  byte slices are raw bytes.

Today every algonaut domain type — `Address`, `MultisigSignature`,
`VotePk`, `VrfPk`, `StateProofPk`, `HashDigest`, `Signature`, plus the
`Bytes` / `serialize_bytes` helpers in `algonaut_encoding` — has a
single `Serialize` impl that calls `serializer.serialize_bytes(&...)`.
That's correct for msgpack (where the bytes become a `bin` value) but
wrong for JSON (where `serde_json` renders bytes as a JSON integer
array). Algod's JSON decoders reject those bodies with errors like:

```
json decode error [pos 142]: decoded bad addr:
```

This surfaced in PR #265 while debugging dryrun against a live
sandbox; the immediate fix there was to re-serialize the request as
msgpack and POST it with `Content-Type: application/x-binary`. The
underlying mismatch is generic — issue
[#271](https://github.com/manuelmauro/algonaut/issues/271) catalogues
the other endpoints it blocks (notably the indexer, which is JSON-only
and has no msgpack escape hatch).

Four options were considered:

1. **Format-aware impls** — branch on `serializer.is_human_readable()`
   inside each `Serialize`/`Deserialize`. JSON path emits canonical
   strings; msgpack path emits raw bytes (unchanged).
2. **Parallel `Api*` newtypes for JSON** — mirror the existing
   `Transaction` ↔ `ApiTransaction` split for every domain type.
3. **Per-field `#[serde(with = "...")]`** annotations on every model.
4. **Force msgpack on every algod endpoint** — doesn't help the
   indexer (JSON-only) and breaks the OpenAPI-generated swagger.

Reference SDKs (java, go, py) all use the equivalent of option 1 —
they pick the format based on the chosen serializer. The Rust
ecosystem precedent (`uuid`, `chrono`, `time`, …) is the same.

## Decision

Adopt **option 1**. The recipe per type, taking `Address` as the
template:

```rust
impl Serialize for Address {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        if s.is_human_readable() {
            s.serialize_str(&self.encode_as_string())
        } else {
            s.serialize_bytes(&self.0[..])
        }
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        if d.is_human_readable() {
            let s = String::deserialize(d)?;
            s.parse().map_err(serde::de::Error::custom)
        } else {
            Ok(Address(d.deserialize_bytes(U8_32Visitor)?))
        }
    }
}
```

Types covered: `Address`, `MultisigAddress`, `MultisigSignature`,
`MultisigSubsig`, `Signature`, `VotePk`, `VrfPk`, `StateProofPk`,
`HashDigest`, plus the `serialize_bytes` / `deserialize_bytes` helpers
in `algonaut_encoding`. `Bytes` (the `Vec<u8>` newtype) renders as
base64 for JSON and as `bin` for msgpack.

Once the change lands, the dryrun-specific msgpack workaround in
`Algod::teal_dryrun` (added in PR #265) can be removed — the generated
JSON client will serialise the body correctly. The same fix unblocks
issue #270 (indexer box queries) and several other cucumber scenarios
that hit JSON-only endpoints.

## Consequences

- **JSON correctness.** The OpenAPI-generated client and any caller
  that uses `serde_json` now produce wire-compatible payloads against
  algod and the indexer. Indexer queries, swagger requests, and the
  remaining JSON-only endpoints stop failing with "bad addr".
- **Msgpack stays byte-identical.** The non-human-readable branch is
  the existing implementation, so transaction-signing tests
  (`test_serialize_signed_logic_contract_account`,
  `test_api_box_references_from_box_references`, etc.) continue to
  pass without changes.
- **One global behaviour change.** Any third-party serializer that
  incorrectly returns `is_human_readable() = true` will hit the
  string path. rmp-serde returns `false`; `serde_json` returns `true`.
  We mitigate the risk with explicit JSON round-trip tests next to
  each updated type.
- **Dryrun workaround is removable.** Keeping it is harmless; we will
  drop it in the same PR for clarity.
- **Future endpoints free.** New JSON-only endpoints (indexer
  expansions, future algod additions) no longer need a per-call
  workaround.
