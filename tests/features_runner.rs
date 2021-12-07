use cucumber::WorldInit;
use step_defs::integration;

mod step_defs;

#[tokio::main]
async fn main() {
    integration::algod::World::run(integration_path("algod")).await;
    integration::abi::World::run(integration_path("abi")).await;
}

fn integration_path(feature_name: &str) -> String {
    format!("tests/features/integration/{}.feature", feature_name)
}
