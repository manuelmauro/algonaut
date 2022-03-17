use algonaut::algod::v2::Algod;
use algonaut::transaction::ClawbackAsset;
use algonaut::transaction::{account::Account, TxnBuilder};
use dotenv::dotenv;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    // The account specified as clawback when creating the asset.
    let sender = Account::from_mnemonic("fire enlist diesel stamp nuclear chunk student stumble call snow flock brush example slab guide choice option recall south kangaroo hundred matrix school above zero")?;
    let sender_address = sender.address();

    // The asset receiver: In this case we'll make the clawback account also the asset receiver.
    let asset_receiver_address = sender_address;

    // The asset "sender": The account from which the asset is withdrawn.
    let asset_sender_address = Account::from_mnemonic("since during average anxiety protect cherry club long lawsuit loan expand embark forum theory winter park twenty ball kangaroo cram burst board host ability left")?.address();

    let params = algod.suggested_transaction_params().await?;

    let t = TxnBuilder::with(
        &params,
        ClawbackAsset::new(
            sender_address,
            4,
            2,
            asset_sender_address,
            asset_receiver_address,
        )
        .build(),
    )
    .build()?;

    let sign_response = sender.sign_transaction(t)?;

    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod.broadcast_signed_transaction(&sign_response).await;
    println!("{:#?}", send_response);

    Ok(())
}
