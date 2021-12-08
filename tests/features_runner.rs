use cucumber::WorldInit;
use step_defs::integration;

mod step_defs;

#[tokio::main]
async fn main() {
    // algod feature: omitted, this tests v1 and we don't support it anymore
    // integration::algod::World::run(integration_path("algod")).await;

    // ABI not supported yet
    // integration::abi::World::run(integration_path("abi")).await;

    integration::applications::World::cucumber()
        .max_concurrent_scenarios(1)
        .run(integration_path("applications"))
        .await;
}

fn integration_path(feature_name: &str) -> String {
    format!("tests/features/integration/{}.feature", feature_name)
}
