use algonaut::atomic_transaction_composer::{AtomicTransactionComposer, AddMethodCallParams, transaction_signer::TransactionSigner};


use std::error::Error;
#[macro_use]

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
 


AtomicTransactionComposer::add_method_call(  
 &self,
 &mut AddMethodCallParams
 );
AtomicTransactionComposer::build_group();
 
AtomicTransactionComposer::execute();
 
}
