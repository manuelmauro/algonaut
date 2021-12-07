use algonaut::algod::v2::Algod;
use algonaut::algod::AlgodBuilder;
use async_trait::async_trait;
use cucumber::{given, WorldInit};
use std::convert::Infallible;

#[derive(Default, Debug, WorldInit)]
pub struct World {
    algod: Option<Algod>,
}

#[async_trait(?Send)]
impl cucumber::World for World {
    type Error = Infallible;

    async fn new() -> Result<Self, Self::Error> {
        Ok(Self::default())
    }
}

#[given(expr = "an algod v2 client")]
async fn an_algod_v2_client(w: &mut World) {
    let algod = AlgodBuilder::new()
        .bind("http://localhost:60000")
        .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        .build_v2()
        .unwrap();
    w.algod = Some(algod)
}
