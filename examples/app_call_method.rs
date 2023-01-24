use algonaut::atomic_transaction_composer::{AtomicTransactionComposer, AddMethodCallParams, transaction_signer::TransactionSigner};
use algonaut::transaction::transaction::{
    ApplicationCallOnComplete,
};
use algonaut::algod::v2::Algod;

use std::error::Error;
#[macro_use]

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
 
 let url = String::from("https://node.testnet.algoexplorerapi.io");
 
 let user = String::from("User-Agent");
 let pass = String::from("DoYouLoveMe?");
 let headers :  Vec<(&str, &str)> = vec![(&user, &pass)];

 let algod = Algod::with_headers(&url, headers)?;
 
 let acct1 = Account::from_mnemonic("degree feature waste gospel screen near subject boost wreck proof caution hen adapt fiber fault level blind entry also embark oval board bunker absorb garage
")?;
 
 println!("retrieving suggested params");
 let params = algod.suggested_transaction_params().await?;
 
 
 let val = String::from("something");
 let arg1 : u64 = 10000; 
 let arg2 = acct1;
 
 
let mut AtomicTransactionComposer = AtomicTransactionComposer::add_method_call(  
 &self,
 &mut AddMethodCallParams{
 app_id: 155672004, method: "withdraw", method_args: [arg1, arg2], fee: TxnFee{Fixed(2000)}, sender: acct1.address(), suggested_params: params, on_complete: ApplicationCallOnComplete{NoOp},
  approval_program: val, clear_program: val, global_schema: val, local_schema: val, extra_pages: val, 
  note: val, lease: val, rekey_to: val, signer: TransactionSigner
 }
 );
AtomicTransactionComposer::build_group(&mut AtomicTransactionComposer);
 
AtomicTransactionComposer::execute( &mut AtomicTransactionComposer ,&algod);
Ok(())
 
 
}
