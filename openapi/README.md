# OpenAPI client regeneration

The `algonaut_algod` and `algonaut_indexer` crates began as openapi-generator
output and have since been **extensively customized by hand** (algonaut domain
types, unsigned integers, the simulate power-pack, serde fixes). They are best
understood as a *maintained fork* of the generated code, not throwaway output.

Per ADR `relocate-generated-models`, the generated **model** structs now live in
`algonaut_model::{algod,indexer}` (the synthesized `<op>200Response` envelopes
renamed to intentional names — see `inlineSchemaNameMappings` in the per-client
configs), while the **api**/transport code stays in the client crates and
imports its models from `algonaut_model`. The three former hand-wrappers
(`SuggestedParams`, `NodeStatus`, `Supply`) are reproduced by the relocated,
renamed and domain-typed models, so they no longer live in
`algonaut_model::client_types` (only the `TransactionParams` trait does).

This directory makes regeneration **reproducible and diff-able** so upstream
drift can be discovered and ported deliberately. See
`docs/adr/openapi-client-regeneration.md` for the full rationale and the staged
plan toward a near-lossless regen.

## Layout

| Path | Purpose |
| --- | --- |
| `specs/algod.oas3.json` | Pinned snapshot of the algod OpenAPI spec. |
| `specs/indexer.oas3.json` | Pinned snapshot of the indexer OpenAPI spec. |
| `config-algod.yaml` | openapi-generator config for the algod client. |
| `config-indexer.yaml` | openapi-generator config for the indexer client. |
| `templates/` | Custom mustache templates (integer/`Bytes`/override emission). |
| `type-overrides.json` | Per-field domain-type override table. |
| `preprocess.py` | Injects the override table into the specs as vendor extensions. |
| `generated/` | Regenerated output (git-ignored; for diffing only). |

## Specs

The pinned snapshots were fetched on 2026-05-19 from:

- algod — `https://raw.githubusercontent.com/algorand/go-algorand/master/daemon/algod/api/algod.oas3.yml`
- indexer — `https://raw.githubusercontent.com/algorand/indexer/main/api/indexer.oas3.yml`

`make fetch-openapi-specs` refreshes them to the latest upstream.

## Workflow

```bash
make generate-clients   # regenerate into openapi/generated/ (needs Docker)
# Review drift — models against algonaut_model, apis against the client crate:
git diff --no-index openapi/generated/algod/src/models   algonaut_model/src/algod
git diff --no-index openapi/generated/algod/src/apis     algonaut_algod/src/apis
git diff --no-index openapi/generated/indexer/src/models algonaut_model/src/indexer
git diff --no-index openapi/generated/indexer/src/apis   algonaut_indexer/src/apis
```

`make generate-clients` never overwrites the crates — it only produces
`openapi/generated/` for review. Adopting upstream changes is a deliberate,
reviewed step.

**Note:** The generator output is *not yet authoritative* — the mustache
templates still emit `crate::models::…` paths instead of `algonaut_model::…`,
and `preprocess.py` does not yet annotate inline response schemas with the
domain-type overrides. Regenerated code therefore requires manual adjustment
before it compiles. The committed source in `algonaut_model` / the client
crates is the maintained artifact; the diff commands above help discover
upstream drift but are not a drop-in copy. Updating the templates to produce
the relocated layout directly is tracked as future work.
