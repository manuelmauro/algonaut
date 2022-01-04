use algonaut::algod::v2::Algod;
use algonaut_core::{LogicSignature, MicroAlgos, MultisigAddress, SignedLogic};
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

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;

    let program = algod
        .compile_teal(
            r#"
#pragma version 3
int 1
"#
            .as_bytes(),
        )
        .await?;

    let account1 = Account::from_mnemonic("fire enlist diesel stamp nuclear chunk student stumble call snow flock brush example slab guide choice option recall south kangaroo hundred matrix school above zero")?;
    let account2 = Account::from_mnemonic("since during average anxiety protect cherry club long lawsuit loan expand embark forum theory winter park twenty ball kangaroo cram burst board host ability left")?;
    let receiver = "DN7MBMCL5JQ3PFUQS7TMX5AH4EEKOBJVDUF4TCV6WERATKFLQF4MQUPZTA".parse()?;

    let multisig_address = MultisigAddress::new(1, 2, &[account1.address(), account2.address()])?;

    let params = algod.suggested_transaction_params().await?;

    let t = TxnBuilder::with(
        params,
        Pay::new(multisig_address.address(), receiver, MicroAlgos(123_456)).build(),
    )
    .build();

    let msig = account1.init_logic_msig(&program, &multisig_address)?;
    let msig = account2.append_to_logic_msig(&program, msig)?;

    let sig = TransactionSignature::Logic(SignedLogic {
        logic: program,
        args: vec![],
        sig: LogicSignature::DelegatedMultiSig(msig),
    });

    let signed_t = SignedTransaction {
        transaction: t,
        transaction_id: "".to_owned(),
        sig,
    };

    let send_response = algod.broadcast_signed_transaction(&signed_t).await;
    println!("response {:?}", send_response);

    Ok(())
}
