use algonaut::algod::v2::Algod;
use algonaut::transaction::account::Account;
use algonaut::transaction::builder::CallApplication;
use algonaut::transaction::TxnBuilder;
use dotenv::dotenv;


use std::error::Error;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();

    let url = String::from("https://node.testnet.algoexplorerapi.io");
    //let token = String::from("");
    
    let alice_mnemonic = String::from("degree feature waste gospel screen near subject boost wreck proof caution hen adapt fiber fault level blind entry also embark oval board bunker absorb garage");
   
    let user = String::from("User-Agent");
    let pass = String::from("DoYouLoveMe?");
    let headers :  Vec<(&str, &str)> = vec![(&user, &pass)];
    
    //= {'User-Agent': 'DoYouLoveMe?}?;
    
    println!("creating algod client");
    //let algod = Algod::new(&url, &token)?;
    let algod = Algod::with_headers(&url, headers)?;
    
    println!("creating account for alice");
    let alice = Account::from_mnemonic(&alice_mnemonic)?;

    println!("retrieving suggested params");
    let params = algod.suggested_transaction_params().await?;

    println!("building transaction");
    let app_id : u64 = 154280672;
    let app_arg : Option<String> = Some(String::from("withdraw"));
    
    //map the string
    //convert each string to bytes via a tuple
    //supply tuple to app call method

    //let arg_as_bytes : Vec<u8> = app_arg.expect("REASON").into_bytes();
    
    //println!("{:?}", &arg_as_bytes);


    
    let t = TxnBuilder::with(
        &params,
        CallApplication::new(alice.address(), app_id)
            .app_arguments(vec![app_arg.expect("REASON").into_bytes()])
            .build(),
    )
    .build()?;

    println!("signing transaction");
    let signed_t = alice.sign_transaction(t)?;

    info!("broadcasting transaction");
    let send_response = algod.broadcast_signed_transaction(&signed_t).await?;
    info!("response: {:?}", send_response);

    Ok(())
}
