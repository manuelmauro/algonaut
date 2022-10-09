use algonaut::algod::v2::Algod;
use algonaut::transaction::{account::Account, TxnBuilder};
use algonaut::util::wait_for_pending_tx::wait_for_pending_transaction;
use algonaut_core::MicroAlgos;
use algonaut_transaction::Pay;
use dotenv::dotenv;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let rekeyed_acc = Account::from_mnemonic("fire enlist diesel stamp nuclear chunk student stumble call snow flock brush example slab guide choice option recall south kangaroo hundred matrix school above zero")?;
    let rekeyed_acc_address = rekeyed_acc.address();

    let rekey_to_acc = Account::from_mnemonic("auction inquiry lava second expand liberty glass involve ginger illness length room item discover ahead table doctor term tackle cement bonus profit right above catch")?;
    let rekey_to_acc_address = rekey_to_acc.address();

    // double check that rekeyed account's auth address is not set
    let account_infos = algod.account_information(&rekeyed_acc_address).await?;
    assert!(account_infos.auth_addr.is_none());

    // rekey
    let params = algod.suggested_transaction_params().await?;
    let rekey_tx = TxnBuilder::with(
        &params,
        Pay::new(rekeyed_acc_address, rekeyed_acc_address, MicroAlgos(0)).build(),
    )
    .rekey_to(rekey_to_acc_address)
    .build()?;
    let rekey_signed = rekeyed_acc.sign_transaction(rekey_tx)?;
    let rekey_response = algod.broadcast_signed_transaction(&rekey_signed).await?;
    wait_for_pending_transaction(&algod, &rekey_response.tx_id).await?;
    println!("Rekey success");

    // verify: rekey_to address is set as auth address of the rekeyed acc
    let account_infos = algod.account_information(&rekeyed_acc_address).await?;
    assert_eq!(Some(rekey_to_acc_address), account_infos.auth_addr);

    // verify: send a tx with the rekeyed address as sender, signing with rekey_to account
    let receiver = "PGCS3D5JL4AIFGTBPDGGMMCT3ODKUUFEFG336MJO25CGBG7ORKVOE3AHSU".parse()?;
    let payment_tx = TxnBuilder::with(
        &params,
        Pay::new(rekeyed_acc_address, receiver, MicroAlgos(10_000)).build(),
    )
    .build()?;
    let payment_signed = rekey_to_acc.sign_transaction(payment_tx)?;
    let payment_response = algod.broadcast_signed_transaction(&payment_signed).await;
    println!("Payment response: {:?}", payment_response);
    assert!(payment_response.is_ok());

    Ok(())
}
