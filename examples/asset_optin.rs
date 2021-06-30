use algonaut::algod::AlgodBuilder;
use algonaut_core::MicroAlgos;
use algonaut_transaction::AcceptAsset;
use algonaut_transaction::{account::Account, TxnBuilder};
use dotenv::dotenv;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v2()?;

    let account = Account::from_mnemonic("since during average anxiety protect cherry club long lawsuit loan expand embark forum theory winter park twenty ball kangaroo cram burst board host ability left")?;

    let params = algod.transaction_params().await?;

    let t = TxnBuilder::new()
        .sender(account.address())
        .first_valid(params.last_round)
        .last_valid(params.last_round + 10)
        .genesis_id(params.genesis_id)
        .genesis_hash(params.genesis_hash)
        .fee(MicroAlgos(100_000))
        .asset_accept(
            AcceptAsset::new()
                .xfer(4)
                .receiver(account.address())
                .build(),
        )
        .build();

    let sign_response = account.sign_transaction(&t);
    println!("{:#?}", sign_response);
    assert!(sign_response.is_ok());
    let sign_response = sign_response.unwrap();

    // Broadcast the transaction to the network
    // Note this transaction will get rejected because the accounts do not have any tokens
    let send_response = algod.broadcast_signed_transaction(&sign_response).await;

    println!("{:#?}", send_response);

    Ok(())
}
