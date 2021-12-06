use algonaut::algod::v2::Algod;
use algonaut::algod::AlgodBuilder;
use algonaut_core::Round;
use algonaut_model::algod::v2::NodeStatus;
use async_trait::async_trait;
use cucumber::{given, then, when, WorldInit};
use std::convert::Infallible;

#[derive(Default, Debug, WorldInit)]
pub struct World {
    algod_client: Option<Algod>,
    node_status: Option<NodeStatus>,
    last_round: Option<u64>,
}

#[async_trait(?Send)]
impl cucumber::World for World {
    type Error = Infallible;

    async fn new() -> Result<Self, Self::Error> {
        Ok(Self::default())
    }
}

#[given(expr = "an algod client")]
async fn an_algod_client(w: &mut World) {
    let algod = AlgodBuilder::new()
        .bind("http://localhost:60000")
        .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        .build_v2()
        .unwrap();

    w.algod_client = Some(algod)
}

#[then(expr = "the node should be healthy")]
async fn node_is_healthy(w: &mut World) {
    let algod_client = w.algod_client.as_ref().unwrap();
    algod_client.health().await.unwrap();
}

#[when(expr = "I get the status")]
async fn i_get_the_status(w: &mut World) {
    let algod_client = w.algod_client.as_ref().unwrap();
    let status = algod_client.status().await.unwrap();
    w.last_round = Some(status.last_round);
    w.node_status = Some(status);
}

#[when(expr = "I get status after this block")]
async fn i_get_the_status_after_this_block(w: &mut World) {
    let algod_client = w.algod_client.as_ref().unwrap();
    let block = w.last_round.unwrap();
    algod_client.status_after_round(Round(block)).await.unwrap();
}

#[then(expr = "I can get the block info")]
async fn i_can_get_the_block_info(w: &mut World) {
    let algod_client = w.algod_client.as_ref().unwrap();
    let last_round = w.last_round.unwrap();
    algod_client.block(Round(last_round)).await.unwrap();
}

#[tokio::main]
async fn main() {
    World::run("tests/features/integration").await;
}
