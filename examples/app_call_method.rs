use algonaut::atomic_transaction_composer::{AtomicTransactionComposer, AddMethodCallParams, transaction_signer::TransactionSigner::BasicAccount};
use algonaut::transaction::{account::Account,
    transaction::ApplicationCallOnComplete::NoOp,
};
use algonaut_transaction::builder::TxnFee::Fixed;
//use algonaut_abi::abi_interactions::AbiReturnType::Void;

use algonaut::atomic_transaction_composer::AbiMethodReturnValue::Void;

use algonaut_abi::abi_interactions::AbiMethod;

use algonaut::core::{CompiledTeal, MicroAlgos};
use algonaut_crypto::HashDigest;
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
 
 
 let val = String::from("");
 let pages: u32 = 0;
    
 //should ideally read from .json file
 let _method : AbiMethod = AbiMethod{
     name: String::from("withdraw"),
     description: Option::<String> = None,
     args: Vec<AbiMethodArg> = vec![AbiMethodArg{name: Option<String> = Some("amount"), description: Option<String> = None,}, 
         AbiMethodArg{name: Option<String> = Some("account"), description: Option<String> = None,},
     ],
     returns: Void,
    }; 
 let arg1 : u64 = 0;
 let arg2 = &acct1.address();
    
 let _note : Option<Vec<u8>> = Some(vec![0]);
 
 
let mut AtomicTransactionComposer = AtomicTransactionComposer::add_method_call(  
 &self,
 &mut AddMethodCallParams{
 app_id: 155672004, method: _method, method_args: [arg1, arg2], fee: TxnFee{Fixed: Fixed(MicroAlgos(2500))}, sender: acct1.address(), suggested_params: params, on_complete: NoOp,
  approval_program: Option<CompiledTeal>, clear_program: Option<CompiledTeal>, global_schema: Option<StateSchema>, local_schema: Option<StateSchema>, extra_pages: pages, 
  note: _note, lease: Option<HashDigest>, rekey_to: Option<Address>, signer: BasicAccount(acct1.mnemonic())
 }
 );
    
//println!("{}",&mut AtomicTransactionComposer);
AtomicTransactionComposer::build_group(&mut AtomicTransactionComposer);
 
AtomicTransactionComposer::execute( &mut AtomicTransactionComposer ,&algod);
Ok(())
 
 
}
