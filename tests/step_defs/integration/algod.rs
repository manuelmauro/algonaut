use crate::step_defs::integration::world::World;
use cucumber::{then, when};

#[then(expr = "the node should be healthy")]
async fn the_node_should_be_healthy(w: &mut World) {
    let algod = w.algod.as_ref().expect("algod not set");
    algod.health().await.expect("health check failed");
}

#[then(expr = "I get the ledger supply")]
async fn i_get_the_ledger_supply(w: &mut World) {
    let algod = w.algod.as_ref().expect("algod not set");
    algod.supply().await.expect("supply call failed");
}

#[when(expr = "I get versions with algod")]
async fn i_get_versions_with_algod(w: &mut World) {
    let algod = w.algod.as_ref().expect("algod not set");
    let version = algod.version().await.expect("version call failed");
    w.versions = Some(version.versions);
}

#[then(regex = r#"^(v\d+) should be in the versions$"#)]
async fn version_should_be_in_the_versions(w: &mut World, expected: String) {
    let versions = w.versions.as_ref().expect("versions not fetched");
    assert!(
        versions.iter().any(|v| v == &expected),
        "expected {expected} in versions, got {versions:?}"
    );
}
