---
id: faster-integration-ci
title: Speed up integration CI by caching the test harness images
abstract: CI wall-clock is gated entirely by the cargo-test-integration job (688s of 688s); within it, ~556s is docker compose build --no-cache of the algod/indexer/conduit sandbox images from source. Take ownership of the harness .env (OVERWRITE_TESTING_ENVIRONMENT=1) and (1) switch algod to a prebuilt channel binary (TYPE=channel) to drop the ~300s go-algorand compile, then (2) stop forcing --no-cache and persist the built images across runs via SHA-pinned actions/cache docker save. Indexer/conduit stay (applications.feature drives the live indexer), and the parallel cargo jobs are left alone.
status: proposed
date: 2026-05-22
deciders: []
tags: [ci, testing, docker, performance]
---

# Speed up integration CI by caching the test harness images

## Status

Proposed. Relates to
[`cucumber-test-suite-coverage-strategy`](cucumber-test-suite-coverage-strategy.md),
which establishes that the runner's feature-set filter is the single source
of truth for *what* runs in CI — this ADR is about how fast *the harness those
features need* comes up, and explicitly does not change coverage.

## Context

A profile of recent `.github/workflows/general.yml` runs (`gh run view
<id> --json jobs`) shows CI wall-clock is gated by exactly one job. On a
warm-cache run of this branch:

| Job | Time |
|---|---|
| **cargo-test-integration** | **688s** |
| cargo-test | 47s |
| cargo-clippy | 30s |
| cargo-check-wasm | 30s |
| cargo-check | 30s |
| cargo-fmt | 9s |
| changes | 5s |

The five fast jobs run in parallel and already finish in under a minute —
`Swatinem/rust-cache@v2` is doing its job. **They are not the bottleneck, so
sharing or improving the cargo cache buys ~0 wall-clock.** CI is the
integration job, and the integration job is the test harness.

### Where the 688s goes

The job's own step timing (harness echoes `seconds it took to finish …`,
buildkit prints `#N DONE Ns`):

- `test-harness.sh up` → `./sandbox up` = **556s**, almost all of it
  `docker compose build` of three images built **from source**.
- The remaining ~130s is `cargo test --test features_runner` (compile +
  cucumber run) and checkout/toolchain/cache restore.

The dominant build steps:

| Step | Time | What |
|---|---|---|
| `[algod] RUN install.sh … -b master` | **299s** | compiles go-algorand from `master` |
| `[algod] exporting to docker image format` | **217s** | exports the algod image |
| `[indexer] RUN /tmp/install.sh` | 77s | builds indexer from `main` |
| `[conduit] RUN /tmp/install.sh` | 73s | builds conduit from `master` |
| indexer/conduit image exports | ~77s | |

### Why "just turn on docker caching" does not work today

The harness `.env` ships with `algorand-sdk-testing@master` and we use it
**as-is**: our `test.env` sets `OVERWRITE_TESTING_ENVIRONMENT=0`, so the
upstream file wins. Two of its settings defeat caching by construction:

1. **`SANDBOX_CLEAN_CACHE=1`** → the sandbox writes a `.clean` marker, and
   its `rebuild_if_needed` then runs `docker compose build **--no-cache**`
   every run. Any layer cache is bypassed deliberately.
2. **`TYPE="source"`** with `ALGOD_BRANCH=master`, `INDEXER_BRANCH=main`,
   `CONDUIT_BRANCH=master` → every run recompiles three Go projects from a
   moving branch tip.

The job already runs `docker/setup-buildx-action@v3`, but nothing wires
buildx's cache backend into the compose build, and `--no-cache` would void
it regardless. GitHub Actions runners are ephemeral, so the local layer
store starts empty each run even if `--no-cache` were dropped — the cache
has to be *persisted* somewhere (gha backend, a registry, or an
`actions/cache` tarball) to survive between runs.

### Constraint: the live indexer cannot be dropped

The obvious "build fewer images" shortcut is out. `applications.feature`
runs ungated (no `gate`/`excluded_*` in `tests/features_runner.rs`) and
drives the **live** indexer:

```gherkin
And I wait for indexer to catch up to the round where my most recent transaction was confirmed.
And according to "indexer", the contents of the box with name "str:name" … should be "AAAA…".
```

The live indexer needs conduit + postgres behind it. Disabling
`INDEXER_DISABLED`/`CONDUIT_DISABLED` would silently drop real coverage,
which `cucumber-test-suite-coverage-strategy` forbids doing without an ADR
and a tracking issue. So all three images stay; the lever is *how fast they
come up*, not *how many*.

## Decision

Take ownership of the harness environment and attack the 556s in two
independent, separately-landable steps. Both require flipping
`test.env`'s `OVERWRITE_TESTING_ENVIRONMENT=1` and committing our own
`.env` (today the upstream one is used verbatim); the integration job is
the only consumer of these settings, so the blast radius is one job.

### 1. Build algod from a prebuilt channel, not from master source

In our `.env`, set `TYPE="channel"` and `ALGOD_CHANNEL="nightly"`. The
algod image then downloads a prebuilt node via `update.sh -c nightly`
instead of compiling go-algorand, removing the **299s** compile and most of
the **217s** export (the export cost is dominated by the freshly-built
binary layer). `nightly` is chosen over `stable`/`beta` because it is the
closest prebuilt analogue to "master tip" — the SDK exercises recent algod
features (simulate, ABI, c2c, box ops) and must not regress to an older
node. Indexer and conduit have no channel equivalent in the sandbox compose
and continue to build from source; step 2 handles them.

This step alone is the high-value, low-risk change: one env file, no new CI
infrastructure, ~300–400s off the critical path.

### 2. Stop forcing `--no-cache` and persist the images across runs

With source builds gone for algod, the remaining indexer/conduit builds
(and algod's downloaded layers) become cacheable — but only if we (a) stop
the forced rebuild and (b) keep the images between ephemeral runs:

- In our `.env`, set **`SANDBOX_CLEAN_CACHE=0`** so the sandbox does not
  write `.clean` and does not pass `--no-cache`.
- **Pin the upstream SHAs** — `INDEXER_SHA`, `CONDUIT_SHA`, and (if any
  source build remains) `ALGOD_SHA` — so a cached layer/image is both
  reusable *and* correct. Building from a moving branch with caching on
  would otherwise silently reuse a stale image: BuildKit keys a `RUN`
  layer on the command string, not on the remote content it fetches, so
  `RUN install.sh -b master` cache-hits even after master advances. Pinning
  makes upstream bumps an explicit, reviewable diff.
- In `.github/workflows/general.yml`, around the harness step, add an
  `actions/cache` step that restores/saves a `docker save` tarball of the
  sandbox images, keyed on the pinned SHAs plus the hash of the sandbox
  Dockerfiles. On a cache hit, `docker load` the tarball before
  `test-harness.sh up`; the subsequent compose build/up reuses the loaded
  images instead of rebuilding. This is the "docker caching" the
  investigation set out to find — it just needs the two upstream blockers
  removed first.

On a warm cache this drives the 556s harness step toward the cost of
`docker load` + `compose up` (tens of seconds). Cold caches (a SHA bump, a
Dockerfile change) pay the build once.

### Phasing and ordering

1. **Step 1 first**, on its own PR — it is the bulk of the win, is trivial
   to review, and de-risks step 2 by shrinking what must be cached.
2. **Step 2 second**, once step 1 is measured in CI. If step 1 alone brings
   the job to an acceptable duration, step 2 can be deferred — it trades
   real complexity (cache key correctness, SHA-bump maintenance) for the
   long tail of indexer/conduit build time.

### Explicitly rejected

- **Sharing/tuning the cargo cache across the fast jobs.** They already
  finish in ≤47s; there is nothing to win.
- **Disabling indexer/conduit.** Loses `applications.feature` coverage (see
  Context).
- **Layer caching without removing `--no-cache`.** A no-op while
  `SANDBOX_CLEAN_CACHE=1` forces a clean rebuild.
- **Caching against the moving `master`/`main` branches.** Would serve
  stale nodes; pinning SHAs is the precondition for caching to be correct.

## Consequences

- **What we test against shifts, deliberately.** algod moves from
  *master source* to the *nightly prebuilt binary*, and (step 2)
  indexer/conduit move from *branch tip* to *pinned SHA*. This is a feature
  for reproducibility — runs stop depending on whatever upstream merged that
  hour — but it means upstream regressions are caught on a lag, and SHA
  bumps become a periodic maintenance chore (a scheduled bump PR, or a
  manual refresh when a new algod feature is needed). Worth a short note in
  `CONTRIBUTING.md`.
- **We now own a copy of the harness `.env`.** Flipping
  `OVERWRITE_TESTING_ENVIRONMENT=1` means our committed `.env` must be kept
  loosely in sync with upstream's when its schema changes (new variables).
  Per the migration-consistency rule this file is part of the change, not a
  follow-up.
- **Critical-path estimate.** Step 1: ~688s → ~300–350s. Step 1 + 2 on a
  warm cache: harness ~556s → tens of seconds, leaving the cargo
  compile/cucumber run (~130s) as the new floor for the integration job.
- **Cold-cache and first-run cost is unchanged-to-slightly-worse** for
  step 2 (the `docker save`/`load` and cache upload add overhead when there
  is nothing to reuse), which is the normal shape of a cache.
- **No coverage change.** The runner's feature set and exclusion filters are
  untouched; this ADR only changes how the harness is provisioned, honoring
  the boundary `cucumber-test-suite-coverage-strategy` drew.
- **Deferred / not decided here.** The exact cache key composition, whether
  to use an `actions/cache` tarball vs pushing the images to GHCR vs a
  buildx `type=gha` backend, and the cadence of SHA-bump PRs are settled
  during implementation of step 2. Whether to pin `ALGOD_SHA` at all (step 1
  removes algod's source build, so there may be nothing to pin there) is
  likewise an implementation detail.
