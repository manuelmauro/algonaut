---
name: prepare-release
description: Prepare a new algonaut version release with changelog and workspace-wide version bumps
license: MIT OR Apache-2.0
---

# Prepare Release

Steps to prepare a new `algonaut` release.

`algonaut` is a Cargo **workspace** of 13 crates (the root `algonaut` library
plus 12 `algonaut_*` members) that are **version-locked**: they all share one
version and ship together. A release therefore bumps every crate, not just one.

## Checklist

1. `make ci` is green. It is fully offline — its cucumber/e2e stages are
   compile-only (`check-cucumber`/`check-e2e` use `cargo test --no-run`), and
   lefthook runs the same `make ci` on every commit. The node-backed *run*
   suites (`make cucumber`, `make test-e2e`) are separate; run them against a
   live node (`make sandbox` or `make harness`) if you want that coverage too
2. `cargo doc` builds (`make doc` — note it only warns; see [Docs](#docs))
3. Bump the version across the whole workspace (all 13 crates + the
   `[workspace.dependencies]` pins) — see [Version Bump](#version-bump)
4. Run `cargo check` to confirm the bumped manifests resolve (`Cargo.lock` is
   gitignored here, so it is not committed)
5. Reconcile `## [Unreleased]` against the git history since the last tag — add
   any notable change that never got an entry (see [Changelog](#changelog))
6. Promote `## [Unreleased]` in `CHANGELOG.md` to the new version
7. Update version references in `README.md` (the "What's new since 0.4.2" list)
8. Commit the bump, changelog, and docs (stage with `git add -A`)
9. Create the annotated `vX.Y.Z` tag
10. Publish the crates to crates.io in dependency order
11. Create the GitHub release for the tag (see [GitHub Release](#github-release))

## Version Bump

The version lives in **two** places per release:

- the `[package] version = "X.Y.Z"` of the root `algonaut` and each of the 12
  `algonaut_*` member crates, and
- the matching `version = "X.Y.Z"` on each path dependency in the root
  `Cargo.toml` `[workspace.dependencies]` table.

These crates do **not** use `version.workspace = true`, so all of these must
move together. Do it in one shot with `cargo-edit`:

```bash
# Bumps every workspace member's [package] version AND the intra-workspace
# dependency requirements that point at them.
cargo set-version --workspace X.Y.Z
```

If `cargo-edit` isn't installed (`cargo install cargo-edit`), bump by hand:
the root `[package].version`, the 12 entries in `[workspace.dependencies]`, and
each member crate's `[package].version`. Verify nothing was missed:

```bash
rg -n '^version = "' Cargo.toml algonaut_*/Cargo.toml
rg -n 'version = "' Cargo.toml   # also catches the [workspace.dependencies] pins
```

Then run `cargo check` to confirm every crate still resolves at the new version:

```bash
cargo check --workspace
```

`Cargo.lock` is gitignored in this repo (line 4 of `.gitignore`), as is usual
for a library workspace — `cargo check` regenerates it locally, but there is
nothing to stage or commit for it.

## Changelog

`CHANGELOG.md` follows [Keep a Changelog](https://keepachangelog.com/) and
SemVer. New work accumulates under `## [Unreleased]` as it lands.

**First, reconcile `[Unreleased]` against the git history since the last tag.**
Entries are written incrementally per PR, so a change can land without one (or a
bullet can describe a later feature as extending a base that was never itself
recorded). Diff the two and fill any gap before promoting the header:

```bash
# Every commit since the last release — map each notable feat/fix/refactor to a
# bullet in [Unreleased]; add the missing ones in the house style below.
git log --oneline "$(git describe --tags --abbrev=0)"..HEAD
```

Watch especially for foundational entries: a bullet that says a feature "now
**also** does X" implies an earlier entry introduced the feature — make sure that
introduction is actually present, and lead the section with it.

Then promote the header — rename `## [Unreleased]` to the version and date and
open a fresh `[Unreleased]` above it:

```markdown
## [Unreleased]

## [X.Y.Z] - YYYY-MM-DD

### Added
- ...

### Changed
- ...

### Removed
- ...
```

Keep the existing house style: each bullet is a self-contained paragraph,
prefix breaking changes with **Breaking:**, and reference the backing ADR
(`docs/adr/<slug>.md`) or PR/issue number where there is one.

## Docs

`make doc` is `cargo doc --no-deps --open` — it does **not** pass `-D warnings`,
so it surfaces rustdoc lints (broken/private intra-doc links) as warnings without
failing. A few such warnings are pre-existing and not release blockers; just make
sure `cargo doc` *builds* (docs.rs needs that) and that the release didn't add new
ones. To see them all, run `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace`.

## Update Version References in Docs

The crates.io / docs.rs / license badges in `README.md` track the published
version automatically — leave them alone. The thing to update by hand is the
**"What's new since 0.4.2"** list: rename the trailing `**unreleased**` bullet
to the new `**X.Y**` line (and add a fresh `**unreleased**` bullet if work has
already started for the next cycle).

Sweep for any other hardcoded versions before committing:

```bash
rg -n 'algonaut.*[0-9]+\.[0-9]+\.[0-9]+|v[0-9]+\.[0-9]+\.[0-9]+' README.md docs/
```

There is no `.github/SECURITY.md` in this repo, so there is no supported-versions
table to maintain.

## Release Commands

```bash
# Full pre-release gate, fully offline (fmt-check, clippy -D warnings, test,
# check-cucumber/check-e2e compile-checks, build) — also lefthook's pre-commit hook
make ci

# Stage EVERYTHING — lefthook lints the staged snapshot, so a partial stage
# lints a tree that won't match what you commit. (Cargo.lock is gitignored.)
git add -A
git commit -m "chore(release): vX.Y.Z"

# Annotated tag, matching the existing vX.Y.Z convention
git tag -a vX.Y.Z -m "Release vX.Y.Z"

# Push commit and tag
git push && git push --tags
```

## Publishing to crates.io

All 13 crates are published together — the umbrella `algonaut` depends on the
members by version, so every one must exist on the registry
(see `docs/adr/publish-entire-workspace.md`). Publish **leaf-first**, so each
crate's dependencies already exist on crates.io when it is published. The
verified topological order is:

```text
algonaut_encoding
algonaut_crypto
algonaut_core
algonaut_abi_sig
algonaut_abi_model
algonaut_abi_macros
algonaut_abi
algonaut_model
algonaut_transaction
algonaut_algod
algonaut_indexer
algonaut_kmd
algonaut          # root crate, last
```

Dry-run a crate before publishing it:

```bash
cargo publish --dry-run -p algonaut_encoding
```

Publish in order. `cargo publish` waits for the index to update, so the next
crate can resolve the one before it:

```bash
for c in algonaut_encoding algonaut_crypto algonaut_core \
         algonaut_abi_sig algonaut_abi_model algonaut_abi_macros algonaut_abi \
         algonaut_model algonaut_transaction \
         algonaut_algod algonaut_indexer algonaut_kmd algonaut; do
  cargo publish -p "$c"
done
```

`cargo workspaces publish` (from `cargo-workspaces`) automates this same
ordered, version-locked publish if you prefer a single command.

## GitHub Release

Every tag gets a matching GitHub release (`gh release list` shows the history).
Match the established style — a short **## Highlights** intro, **### Key
Features** and **### Breaking Changes** bullet lists, and a link to the changelog
section — and mark it latest:

```bash
gh release create vX.Y.Z --title "vX.Y.Z" --notes-file notes.md --latest
```

The changelog link anchor is the header lowercased with brackets/dots/spaces
stripped, e.g. `[0.9.0] - 2026-06-11` →
`CHANGELOG.md#090---2026-06-11`.
