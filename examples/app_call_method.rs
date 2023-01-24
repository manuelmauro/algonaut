use algonaut::atomic_transaction_composer::{AtomicTransactionComposer,transaction_signer::TransactionSigner}



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
 
 let app_id: u64 = 01;
 let method: AbiMethod = "";
 let method_args: Vec<AbiArgValue> = "";
 let fee: TxnFee = "";
 let sender: Address = "";
 let suggested_params: SuggestedTransactionParams = "";
 let on_complete: ApplicationCallOnComplete ="";
 let approval_program: Option<CompiledTeal> ="";
 let clear_program: Option<CompiledTeal> ="";
 let global_schema: Option<StateSchema> ="";
 let local_schema: Option<StateSchema> =""; 
 let extra_pages: u32 = 123;
 let note: Option<Vec<u8>> ="";
 let lease: Option<HashDigest> ="";
 let rekey_to: Option<Address> ="";
 let signer: TransactionSigner = "";

AtomicTransactionComposer::add_method_call(  
  )
AtomicTransactionComposer::build_group()
 
AtomicTransactionComposer::execute()
 
}
