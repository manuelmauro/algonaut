use algonaut::algod::AlgodBuilder;
use algonaut_core::{MicroAlgos, MultisigAddress};
use algonaut_transaction::transaction::TransactionSignature;
use algonaut_transaction::{account::Account, TxnBuilder};
use algonaut_transaction::{Pay, SignedTransaction};
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

    let account1 = Account::from_mnemonic("fire enlist diesel stamp nuclear chunk student stumble call snow flock brush example slab guide choice option recall south kangaroo hundred matrix school above zero")?;
    let account2 = Account::from_mnemonic("since during average anxiety protect cherry club long lawsuit loan expand embark forum theory winter park twenty ball kangaroo cram burst board host ability left")?;

    let multisig_address = MultisigAddress::new(1, 2, &[account1.address(), account2.address()])?;

    let params = algod.suggested_transaction_params().await?;

    let t = TxnBuilder::with(
        params,
        Pay::new(
            multisig_address.address(),
            account2.address(),
            MicroAlgos(123_456),
        )
        .build(),
    )
    .build();

    let msig = account1.init_transaction_msig(&t, &multisig_address)?;
    let msig = account2.append_to_transaction_msig(&t, msig)?;

    let sig = TransactionSignature::Multi(msig);

    let signed_t = SignedTransaction {
        transaction: t,
        transaction_id: "".to_owned(),
        sig,
    };

    let send_response = algod.broadcast_signed_transaction(&signed_t).await;
    println!("{:#?}", send_response);

    Ok(())
}
