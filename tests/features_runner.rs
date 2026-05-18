//! Cucumber acceptance runner.
//!
//! The Algorand cross-SDK acceptance suite is sourced from
//! [`algorand-sdk-testing`](https://github.com/algorand/algorand-sdk-testing).
//! `./test-harness.sh up` clones the harness and copies its features into
//! `tests/features/{integration,unit}` (both directories are gitignored).
//!
//! Coverage is tracked via ADRs under `docs/adr/` — see
//! `cucumber-test-suite-coverage-strategy` for the overarching plan and
//! `docs/cucumber-pending-issues.md` for the drafted upstream tickets.
//!
//! This runner lists every `.feature` file in the suite. Each entry is
//! either:
//!
//! - **Live** (`gate: None`) — wired to a step-def module and exercised
//!   by CI. Individual scenarios can still be excluded with
//!   `excluded_tags` while the rest of the feature runs.
//! - **Stubbed** (`gate: Some(reason)`) — listed for visibility, skipped
//!   at runtime until the gating ADR lands.
//!
//! Adding a feature: drop a step-def module under `tests/step_defs/`,
//! flip the corresponding entry's `gate` to `None`, and remove the ADR
//! reference.

use cucumber::World;
use step_defs::integration;

mod step_defs;

/// One row in the coverage matrix. The `gate` documents *why* a feature
/// is not yet live; CI ignores stubbed entries.
struct Feature {
    /// Path relative to the workspace root.
    path: &'static str,
    /// `None` => live, `Some(reason)` => skipped with rationale.
    gate: Option<&'static str>,
    /// Scenario tags to *exclude* from the run (e.g. ones blocked on an
    /// ADR while the rest of the feature is live).
    excluded_tags: &'static [&'static str],
}

const INTEGRATION_FEATURES: &[Feature] = &[
    Feature {
        path: "tests/features/integration/applications.feature",
        gate: None,
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/integration/abi.feature",
        gate: None,
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/integration/c2c.feature",
        gate: None,
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/integration/algod.feature",
        gate: None,
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/integration/compile.feature",
        gate: None,
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/integration/assets.feature",
        gate: None,
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/integration/auction.feature",
        gate: None,
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/integration/dryrun.feature",
        gate: Some("blocked on ADR dryrun-request-builder"),
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/integration/dryrun_testing.feature",
        gate: Some("blocked on ADR dryrun-request-builder"),
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/integration/kmd.feature",
        gate: None,
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/integration/rekey.feature",
        gate: None,
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/integration/send.feature",
        gate: None,
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/integration/simulate.feature",
        gate: Some(
            "blocked on ADRs simulaterequest-model-needs-power-pack-fields and \
             atomictransactioncomposer-simulate-convenience",
        ),
        excluded_tags: &[],
    },
];

const UNIT_FEATURES: &[Feature] = &[
    Feature {
        path: "tests/features/unit/abijson.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/unit/algodclient_paths.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/unit/atomic_transaction_composer.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/unit/client-no-headers.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/unit/dryrun_trace.feature",
        gate: Some("blocked on ADRs dryrun-request-builder and cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/unit/feetest.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/unit/offline.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/unit/program_sanity_check.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/unit/rekey.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/unit/responses.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/unit/sourcemap.feature",
        gate: Some("blocked on ADRs teal-source-map-decoder and cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/unit/tealsign.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/unit/transactions.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/unit/v2algodclient_paths.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/unit/v2algodclient_responses.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/unit/v2indexerclient_paths.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
    },
    Feature {
        path: "tests/features/unit/v2indexerclient_responses.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
    },
];

async fn run_integration(path: &str, excluded_tags: &'static [&'static str]) {
    integration::world::World::cucumber()
        .max_concurrent_scenarios(1)
        .filter_run(path, move |_, _, sc| {
            !sc.tags.iter().any(|t| excluded_tags.iter().any(|x| x == t))
        })
        .await;
}

#[tokio::main]
async fn main() {
    let mut skipped: Vec<(&str, &str)> = Vec::new();

    for feature in INTEGRATION_FEATURES.iter().chain(UNIT_FEATURES.iter()) {
        match feature.gate {
            None => run_integration(feature.path, feature.excluded_tags).await,
            Some(reason) => skipped.push((feature.path, reason)),
        }
    }

    if !skipped.is_empty() {
        eprintln!("\nSkipped features (see docs/adr/ for status):");
        for (path, reason) in skipped {
            eprintln!("  - {path}\n      {reason}");
        }
    }
}
