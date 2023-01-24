use algonaut::atomic_transaction_composer::{AtomicTransactionComposer, AddMethodCallParams, transaction_signer::TransactionSigner};
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
 
 let val = String::from("something");
 
let mut AtomicTransactionComposer = AtomicTransactionComposer::add_method_call(  
 &self,
 &mut AddMethodCallParams{
 app_id: val, method: val, method_args: val, fee: val, sender: val, suggested_params: val, on_complete: val,
  approval_program: val, clear_program: val, global_schema: val, local_schema: val, extra_pages: val, 
  note: val, lease: val, rekey_to: val, signer: val 
 }
 );
AtomicTransactionComposer::build_group(&mut AtomicTransactionComposer);
 
AtomicTransactionComposer::execute( &mut AtomicTransactionComposer ,&algod);;
Ok(())
 
 
}
