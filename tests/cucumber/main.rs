//! Cucumber acceptance runner.
//!
//! The Algorand cross-SDK acceptance suite is sourced from
//! [`algorand-sdk-testing`](https://github.com/algorand/algorand-sdk-testing).
//! `./test-harness.sh up` clones the harness and copies its features into
//! `tests/cucumber/features/{integration,unit}` (both directories are
//! gitignored).
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
//! Adding a feature: drop a step-def module under `tests/cucumber/step_defs/`,
//! flip the corresponding entry's `gate` to `None`, and remove the ADR
//! reference.

use cucumber::World;
use cucumber::writer::Stats;
use step_defs::{integration, unit};

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
    /// Scenario *names* to exclude. Use this when an individual scenario
    /// must be skipped but shares its tag with scenarios that stay live.
    excluded_scenarios: &'static [&'static str],
}

const INTEGRATION_FEATURES: &[Feature] = &[
    Feature {
        path: "tests/cucumber/features/integration/applications.feature",
        gate: None,
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/integration/abi.feature",
        gate: None,
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/integration/c2c.feature",
        gate: None,
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/integration/algod.feature",
        gate: None,
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/integration/compile.feature",
        gate: None,
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/integration/assets.feature",
        gate: None,
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/integration/auction.feature",
        gate: None,
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/integration/dryrun.feature",
        gate: None,
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/integration/dryrun_testing.feature",
        gate: None,
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/integration/kmd.feature",
        gate: None,
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/integration/rekey.feature",
        gate: None,
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/integration/send.feature",
        gate: None,
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/integration/simulate.feature",
        gate: None,
        // `simulate.exec_trace_with_stack_scratch` trips the same ATC
        // base64-decode issue tracked in #266; the
        // `simulate.exec_trace_with_state_change_and_hash` scenarios
        // need a `create-and-optin` on-complete combo on the
        // `CreateApplication` builder that we haven't added yet.
        excluded_tags: &[
            "simulate.exec_trace_with_stack_scratch",
            "simulate.exec_trace_with_state_change_and_hash",
        ],
        excluded_scenarios: &[],
    },
];

const UNIT_FEATURES: &[Feature] = &[
    Feature {
        path: "tests/cucumber/features/unit/abijson.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/unit/algodclient_paths.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/unit/atomic_transaction_composer.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/unit/client-no-headers.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/unit/dryrun_trace.feature",
        gate: Some("blocked on ADRs dryrun-request-builder and cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/unit/feetest.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/unit/offline.feature",
        gate: None,
        // Only the address/mnemonic/microalgos round-trip scenarios are
        // wired so far. The transaction-signing scenarios share step
        // phrases (mnemonic for private key, multisig addresses, etc.)
        // with the integration features; wiring them to the unit world
        // needs care to avoid duplicate-step ambiguity. Tracked as
        // follow-up under ADR `cucumber-unit-test-scaffolding`.
        excluded_tags: &[
            "unit.offline.sign",
            "unit.offline.signMsig",
            "unit.offline.signFlat",
            "unit.offline.signFlatLogic",
            "unit.offline.appendMsig",
            "unit.offline.mergeMsig",
        ],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/unit/program_sanity_check.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/unit/rekey.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/unit/responses.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/unit/sourcemap.feature",
        gate: Some("blocked on ADRs teal-source-map-decoder and cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/unit/tealsign.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/unit/transactions.feature",
        gate: Some("blocked on ADR cucumber-unit-test-scaffolding"),
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/unit/v2algodclient_paths.feature",
        gate: None,
        excluded_tags: &[],
        // Excluded scenarios, each for a concrete capability gap — never an
        // assertion weakening. Query-parameter *ordering* is no longer a
        // reason to exclude: `expect the path used to be` compares the query
        // string as an unordered set (RFC 3986).
        // - "Get Block, header-only": the generated `get_block` endpoint has
        //   no `header-only` query parameter (only `format`), so that path
        //   cannot be built without a generated-crate change.
        excluded_scenarios: &["Get Block, header-only"],
    },
    Feature {
        path: "tests/cucumber/features/unit/v2algodclient_responses.feature",
        gate: None,
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
    Feature {
        path: "tests/cucumber/features/unit/v2indexerclient_paths.feature",
        gate: None,
        excluded_tags: &[],
        // Excluded scenarios, each for a concrete capability gap — never an
        // assertion weakening. Query-parameter *ordering* is no longer a
        // reason to exclude: `expect the path used to be` compares the query
        // string as an unordered set (RFC 3986).
        // - "SearchAccounts path with OnlineOnly": the generated
        //   `search_for_accounts` operation has no `online-only` query
        //   parameter, so that path cannot be built without a
        //   generated-crate change.
        // - "SearchForTransactions path": one row exercises a `group-id`
        //   query parameter that the generated `search_for_transactions`
        //   operation does not expose, so that path cannot be built without
        //   a generated-crate change.
        excluded_scenarios: &[
            "SearchAccounts path with OnlineOnly",
            "SearchForTransactions path",
        ],
    },
    Feature {
        path: "tests/cucumber/features/unit/v2indexerclient_responses.feature",
        gate: None,
        excluded_tags: &[],
        excluded_scenarios: &[],
    },
];

/// Decide whether a scenario should run, given the feature's exclusion lists.
///
/// An `excluded_scenarios` entry is matched against the scenario name. To
/// disambiguate two `Scenario Outline`s that share a name (the upstream
/// fixtures do this), an entry may be written as `"<tag>::<name>"`, which
/// only matches a scenario carrying that exact tag.
fn scenario_enabled(
    sc: &cucumber::gherkin::Scenario,
    excluded_tags: &[&str],
    excluded_scenarios: &[&str],
) -> bool {
    if sc.tags.iter().any(|t| excluded_tags.contains(&t.as_str())) {
        return false;
    }
    for entry in excluded_scenarios {
        match entry.split_once("::") {
            Some((tag, name)) => {
                if sc.name == name && sc.tags.iter().any(|t| t == tag) {
                    return false;
                }
            }
            None => {
                if sc.name == *entry {
                    return false;
                }
            }
        }
    }
    true
}

/// Report a feature run's failures and return whether it had any.
///
/// `cucumber`'s `filter_run` *reports* failed/errored steps but never fails the
/// process, so without this check `cargo test --test cucumber` exits 0 even when
/// scenarios fail — and CI lights green on a red suite.
fn run_had_failures<Wld, Wr: Stats<Wld>>(path: &str, writer: &Wr) -> bool {
    if writer.execution_has_failed() {
        eprintln!(
            "FAILED {path}: {} failed step(s), {} parsing error(s), {} hook error(s)",
            writer.failed_steps(),
            writer.parsing_errors(),
            writer.hook_errors(),
        );
        true
    } else {
        false
    }
}

async fn run_integration(
    path: &str,
    excluded_tags: &'static [&'static str],
    excluded_scenarios: &'static [&'static str],
) -> bool {
    let writer = integration::world::World::cucumber()
        .max_concurrent_scenarios(1)
        .filter_run(path, move |_, _, sc| {
            scenario_enabled(sc, excluded_tags, excluded_scenarios)
        })
        .await;
    run_had_failures(path, &writer)
}

async fn run_unit(
    path: &str,
    excluded_tags: &'static [&'static str],
    excluded_scenarios: &'static [&'static str],
) -> bool {
    let writer = unit::world::UnitWorld::cucumber()
        .max_concurrent_scenarios(1)
        .filter_run(path, move |_, _, sc| {
            scenario_enabled(sc, excluded_tags, excluded_scenarios)
        })
        .await;
    run_had_failures(path, &writer)
}

#[tokio::main]
async fn main() {
    let mut skipped: Vec<(&str, &str)> = Vec::new();
    let mut any_failed = false;

    for feature in INTEGRATION_FEATURES {
        match feature.gate {
            None => {
                any_failed |= run_integration(
                    feature.path,
                    feature.excluded_tags,
                    feature.excluded_scenarios,
                )
                .await;
            }
            Some(reason) => skipped.push((feature.path, reason)),
        }
    }

    for feature in UNIT_FEATURES {
        match feature.gate {
            None => {
                any_failed |= run_unit(
                    feature.path,
                    feature.excluded_tags,
                    feature.excluded_scenarios,
                )
                .await;
            }
            Some(reason) => skipped.push((feature.path, reason)),
        }
    }

    if !skipped.is_empty() {
        eprintln!("\nSkipped features (see docs/adr/ for status):");
        for (path, reason) in skipped {
            eprintln!("  - {path}\n      {reason}");
        }
    }

    // `filter_run` reports failures but does not fail the process; propagate a
    // non-zero exit so `cargo test --test cucumber` (and the CI job) goes red
    // when any scenario fails, instead of lighting green on a red suite.
    if any_failed {
        eprintln!("\nintegration suite FAILED — see the FAILED lines above");
        std::process::exit(1);
    }
}
