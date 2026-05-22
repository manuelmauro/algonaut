use algonaut::algod::v2::Algod;
use algonaut::transaction::account::Account;
use algonaut_core::MicroAlgos;
use algonaut_transaction::Pay;
use dotenv::dotenv;
use std::env;
use std::error::Error;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();

    info!("creating algod client");
    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    info!("creating rekey-ed account");
    let rekeyed_acc = Account::from_mnemonic(
        "fire enlist diesel stamp nuclear chunk student stumble call snow flock brush example slab guide choice option recall south kangaroo hundred matrix school above zero",
    )?;
    let rekeyed_acc_address = rekeyed_acc.address();

    info!("creating rekey-to account");
    let rekey_to_acc = Account::from_mnemonic(
        "auction inquiry lava second expand liberty glass involve ginger illness length room item discover ahead table doctor term tackle cement bonus profit right above catch",
    )?;
    let rekey_to_acc_address = rekey_to_acc.address();

    info!("checking auth address");
    // double check that rekeyed account's auth address is not set
    let account_infos = algod.account(&rekeyed_acc_address).await?;
    assert!(account_infos.auth_addr.is_none());

    info!("retrieving suggested params");
    let params = algod.suggested_params().await?;

    info!("creating rekey-ing transaction");
    let rekey_tx = Pay::rekey(rekeyed_acc_address, rekey_to_acc_address).build(&params)?;

    info!("signing transaction");
    let rekey_signed = rekeyed_acc.sign(rekey_tx)?;

    info!("broadcasting transaction");
    algod.submit(&rekey_signed).await?.confirm().await?;
    info!("rekey success");

    info!("verifying the rekey success");
    // verify: rekey_to address is set as auth address of the rekeyed acc
    let account_infos = algod.account(&rekeyed_acc_address).await?;
    assert_eq!(
        Some(rekey_to_acc_address.to_string()),
        account_infos.auth_addr
    );

    info!("testing the rekey success");
    // verify: send a tx with the rekeyed address as sender, signing with rekey_to account
    let receiver = "PGCS3D5JL4AIFGTBPDGGMMCT3ODKUUFEFG336MJO25CGBG7ORKVOE3AHSU".parse()?;
    let payment_tx = Pay::new(rekeyed_acc_address, receiver, MicroAlgos(10_000)).build(&params)?;

    info!("signing transaction");
    let payment_signed = rekey_to_acc.sign(payment_tx)?;

    info!("broadcasting transaction");
    let payment_response = algod.send(&payment_signed).await;
    info!("payment response: {:?}", payment_response);
    assert!(payment_response.is_ok());

    Ok(())
}
