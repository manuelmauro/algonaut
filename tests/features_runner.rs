use cucumber::WorldInit;
use step_defs::integration;

mod step_defs;

#[tokio::main]
async fn main() {
    // NOTE: we don't support algod v1 anymore
    // features which depend completely on algod v1 are omitted

    // algod feature: omitted (algod v1)
    // assets feature: omitted (algod v1)

    // TODO use tags - so we don't have to create a new config per file (until the tests are complete)

    integration::world::World::cucumber()
        .max_concurrent_scenarios(1)
        // show output (e.g. println! or dbg!) in terminal https://cucumber-rs.github.io/cucumber/current/output/terminal.html#manual-printing
        // .with_writer(
        //     writer::Basic::raw(io::stdout(), writer::Coloring::Auto, 0)
        //         .summarized()
        //         .assert_normalized(),
        // )
        .run("tests/features/integration/applications.feature")
        .await;

    integration::world::World::cucumber()
        .max_concurrent_scenarios(1)
        .run("tests/features/integration/abi.feature")
        .await;

    integration::world::World::cucumber()
        .max_concurrent_scenarios(1)
        .run("tests/features/integration/c2c.feature")
        .await;
}
