# OpenAPI client regeneration

The `algonaut_algod` and `algonaut_indexer` crates began as openapi-generator
output and have since been **extensively customized by hand** (algonaut domain
types, unsigned integers, the simulate power-pack, serde fixes). They are best
understood as a *maintained fork* of the generated code, not throwaway output.

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
git diff --no-index openapi/generated/algod/src algonaut_algod/src   # review drift
```

`make generate-clients` never overwrites the crates — it only produces
`openapi/generated/` for review. Adopting upstream changes is a deliberate,
reviewed step.
