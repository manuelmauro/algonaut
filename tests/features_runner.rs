use cucumber::WorldInit;
use step_defs::integration;

mod step_defs;

#[tokio::main]
async fn main() {
    // NOTE: we don't support algod v1 anymore
    // features which depend completely on algod v1 are omitted

    // algod feature: omitted (algod v1)

    // TODO abi feature: ABI not supported yet

    integration::applications::World::cucumber()
        .max_concurrent_scenarios(1)
        .run(integration_path("applications"))
        .await;

    // assets feature: omitted (algod v1)
}

fn integration_path(feature_name: &str) -> String {
    format!("tests/features/integration/{}.feature", feature_name)
}
