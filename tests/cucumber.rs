use cucumber::World;
use step_defs::world;

mod step_defs;

#[tokio::main]
async fn main() {
    // NOTE: we don't support algod v1 anymore
    // features which depend completely on algod v1 are omitted

    // algod feature: omitted (algod v1)
    // assets feature: omitted (algod v1)

    // TODO use tags - so we don't have to create a new config per file (until the tests are complete)

    world::World::cucumber()
        .max_concurrent_scenarios(1)
        .fail_on_skipped()
        .run_and_exit("tests/features/integration/applications.feature")
        .await;

    world::World::cucumber()
        .max_concurrent_scenarios(1)
        .fail_on_skipped()
        .run_and_exit("tests/features/integration/abi.feature")
        .await;

    world::World::cucumber()
        .max_concurrent_scenarios(1)
        .fail_on_skipped()
        .run_and_exit("tests/features/integration/c2c.feature")
        .await;
}
