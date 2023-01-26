use algonaut::atomic_transaction_composer::{AtomicTransactionComposer, AddMethodCallParams, transaction_signer::TransactionSigner::BasicAccount};
use algonaut::transaction::{account::Account, Pay, TxnBuilder,
    transaction::ApplicationCallOnComplete::NoOp,
};
use algonaut_transaction::builder::TxnFee::Fixed;

use algonaut_abi::abi_interactions::AbiReturnType::Void;
use algonaut_abi::abi_type::AbiType;
//
//use algonaut::atomic_transaction_composer::AbiMethodReturnValue::Void;
use algonaut_abi::abi_type::AbiValue as OtherAbiValue;
use algonaut::atomic_transaction_composer::{AbiArgValue, AbiArgValue::AbiValue};
use algonaut_abi::abi_interactions::AbiArgType;
use algonaut_abi::abi_interactions::{AbiMethod,AbiMethodArg,AbiReturn};
use num_bigint::BigUint;
use algonaut_abi::abi_type::AbiType::Address;
use algonaut_abi::abi_type::AbiValue::Int;
use algonaut::core::{CompiledTeal, MicroAlgos};

use algonaut_crypto::HashDigest;
use algonaut::algod::v2::Algod;

use std::error::Error;
use algonaut::atomic_transaction_composer::AtomicTransactionComposerStatus::Building;
use algonaut::atomic_transaction_composer::TransactionWithSigner;

use std::collections::HashMap;

//use std::collections::hash_map::HashMap;

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
 
  let acct1_2 = Account::from_mnemonic("degree feature waste gospel screen near subject boost wreck proof caution hen adapt fiber fault level blind entry also embark oval board bunker absorb garage
")?;   
 println!("retrieving suggested params");
 let params = algod.suggested_transaction_params().await?;
 let params2 = algod.suggested_transaction_params().await?;
 
 let val1 = String::from("val");
 let val2 = String::from("val");
 let val3 = String::from("val");
 let val4 = String::from("val");
 
 let pages: u32 = 0;
  
 let method_name1 : Option<String> = Some("amount".to_string());
 let method_name2 : Option<String> = Some("account".to_string());
 let type1 : String = String::from("uint64");
 let type2 : String = String::from("Address");
 let description1 : Option<String> = Some("amount description".to_string());
 let description2 : Option<String> = Some("account description".to_string());
 let description3 : Option<String> = Some("misc description".to_string());
    
    
 let method_name1_2 : Option<String> = Some("amount".to_string());
 let method_name2_2 : Option<String> = Some("account".to_string());
    
 let type1_2 : String = String::from("uint64");
 let type2_2 : String = String::from("Address");
    
 let description1_2 : Option<String> = Some("amount description".to_string());
 let description2_2 : Option<String> = Some("account description".to_string());
 let description3_2 : Option<String> = Some("misc description".to_string());
 //let mut _signer = BasicAccount(acct1);
 //should ideally read from .json file
    
 let method_arg1 :  AbiMethodArg = AbiMethodArg {
             name: method_name2_2,
             //type_: type2_2,
             description: description3_2,
             //parsed: None
         },
    
    
 method_arg.type_(AbiArgType { AbiObj(AbiType {UInt {
        bit_size: u16,
    }})});   
    
 let _method : AbiMethod = AbiMethod {
     name: String::from("withdraw"),
     description: description1,
     args: vec![
         AbiMethodArg {
             name: method_name1,
             type_: type1,
             description: description2,
             parsed: None
         },
         AbiMethodArg {
             name: method_name2,
             type_: type2,
             description: description3,
             parsed: None
         },
     ],
     returns: AbiReturn {
         type_: val1,
         description: Some(val2),
         parsed: None
     },
 };
  //Duplicate of Method 1 without clone(),
  let _method2 : AbiMethod = AbiMethod {
     name: String::from("withdraw"),
     description: description1_2,
     args: vec![
         AbiMethodArg {
             name: method_name1_2,
             type_: type1_2,
             description: description2_2,
             parsed: None
         },
         AbiMethodArg {
             name: method_name2_2,
             type_: type2_2,
             description: description3_2,
             parsed: None
         },
     ],
     returns: AbiReturn {
         type_: val3,
         description: Some(val4),
         parsed: None
     },
 };

 //https://docs.rs/num-bigint/0.4.3/num_bigint/struct.BigUint.html
 let withdrw_amt : BigUint = BigUint::new(vec![0]);
 let withdrw_to_addr : BigUint = BigUint::new(vec![0]);
 let arg1 : AbiArgValue = AbiArgValue::AbiValue( Int(withdrw_amt));
 let arg2 : AbiArgValue = AbiArgValue::AbiValue( Int(withdrw_to_addr));// &acct1.address();

 const Q : usize = 0usize;

 let mut _hashmap: HashMap<usize, AbiMethod>= std::collections::HashMap::new(); //HashMap<usize, AbiMethod>




 
 _hashmap.insert(Q,_method2); 
 //_hashmap.insert(q,q);
    
 let _note : Option<Vec<u8>> = Some(vec![0]);
//println!("building Pay transaction");
 let t = TxnBuilder::with(

        &params2,

        Pay::new(acct1.address(), acct1.address(), MicroAlgos(123_456)).build(),

    )

    .build();
let t2 = t.unwrap().clone();
let t3 = t2.clone();
let sign_txn = acct1.sign_transaction(t2)?;

let mut atc2 = AtomicTransactionComposer::add_method_call(
        &mut AtomicTransactionComposer {
        status: Building,
        method_map: _hashmap,
        txs: vec![
            TransactionWithSigner {
                tx: t3,
                signer : BasicAccount(acct1)
            }
        ],
        signed_txs: vec![sign_txn],
        },
        &mut AddMethodCallParams {
                 app_id: 155672004,
                 method: _method,
                 method_args: vec![arg1, arg2],
                 fee: Fixed(MicroAlgos(2500)),
                 sender: acct1_2.address(),
                 suggested_params: params,
                 on_complete: NoOp,
                 approval_program: None,
                 clear_program: None,
                 global_schema: None,
                 local_schema: None,
                 extra_pages: pages,
                 note: _note,
                 lease: None,
                 rekey_to: None,
                 signer: BasicAccount(acct1_2)
         
    }
);

//println!("{}",&mut AtomicTransactionComposer);
//AtomicTransactionComposer::build_group(&mut ATC.unwrap());
 
//AtomicTransactionComposer::execute( &mut ATC ,&algod);
Ok(())
 
}

