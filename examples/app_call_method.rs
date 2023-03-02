//code reference: https://github.com/manuelmauro/algonaut/blob/main/tests/step_defs/integration/abi.rs

use algonaut::atomic_transaction_composer::{ transaction_signer::TransactionSigner::BasicAccount};
use algonaut::transaction::{account::Account,
    transaction::ApplicationCallOnComplete::NoOp,
};
use algonaut_transaction::builder::TxnFee::Fixed;

use algonaut_abi::abi_interactions::AbiReturnType::Void;

//
//use algonaut::atomic_transaction_composer::AbiMethodReturnValue::Void;

use algonaut::atomic_transaction_composer::{ AbiArgValue::AbiValue};

use algonaut_abi::abi_interactions::{AbiMethodArg};
use num_bigint::BigUint;
use algonaut_abi::abi_type::AbiType::Address;
use algonaut_abi::abi_type::AbiValue::Int;


use algonaut_crypto::HashDigest;
use algonaut::algod::v2::Algod;


use algonaut::atomic_transaction_composer::AtomicTransactionComposerStatus::Building;

use std::collections::HashMap;

//use std::collections::hash_map::HashMap;
//use crate::step_defs::{
//    integration::world::World,
//    util::{read_teal, wait_for_pending_transaction},
// };
use algonaut::{
    atomic_transaction_composer::{
        transaction_signer::TransactionSigner, AbiArgValue, AbiMethodReturnValue,
        AbiReturnDecodeError, AddMethodCallParams, AtomicTransactionComposer,
        AtomicTransactionComposerStatus, TransactionWithSigner,
    },
    error::ServiceError,
};
use algonaut_abi::{
    abi_interactions::{AbiArgType, AbiMethod, AbiReturn, AbiReturnType, ReferenceArgType},
    abi_type::{AbiType, AbiValue as OtherAbiValue},
};
use algonaut_core::{to_app_address, Address as OtherAddress, MicroAlgos, CompiledTeal};
use algonaut_model::algod::v2::PendingTransaction;
use algonaut_transaction::{
    builder::TxnFee,
    transaction::{ApplicationCallOnComplete, StateSchema},
    Pay, TxnBuilder,
};
//use cucumber::{codegen::Regex, given, then, when};
use data_encoding::BASE64;
use num_traits::ToPrimitive;

//use sha2::Digest;
//use std::convert::TryInto;
use std::error::Error;



#[macro_use]

#[tokio::main]

// #[derive(Clone)]
async fn main() -> Result<(), Box<dyn Error>> {
 
 let url = String::from("https://node.testnet.algoexplorerapi.io");
 
 let user = String::from("User-Agent");
 let pass = String::from("DoYouLoveMe?");
 let headers :  Vec<(&str, &str)> = vec![(&user, &pass)];

 let algod = Algod::with_headers(&url, headers)?;
 
 let acct1 = Account::from_mnemonic("degree feature waste gospel screen near subject boost wreck proof caution hen adapt fiber fault level blind entry also embark oval board bunker absorb garage")?;
 
 let acct1_2 = acct1.clone();
    
 let acct1_3 = acct1.clone();
  

 println!("retrieving suggested params");
 let params = algod.suggested_transaction_params().await?;
 let params2 = params.clone();
 
 //let val1 = String::from("val");
 //let val2 = String::from("val");
 //let val3 = String::from("val");
 //let val4 = String::from("val");
 
 let pages: u32 = 0;
  
 //let method_name1 : Option<String> = Some("amount".to_string());
 //let method_name2 : Option<String> = Some("account".to_string());
 
 //let type1 : String = String::from("uint64");
 //let type2 : String = String::from("Address");
 
 //let description1 : Option<String> = Some("amount description".to_string());
 //let description2 : Option<String> = Some("account description".to_string());
 //let description3 : Option<String> = Some("misc description".to_string());
    
    
 //let method_name1_2 : Option<String> = Some("amount".to_string());
 //let method_name2_2 : Option<String> = Some("account".to_string());
    
 //let type1_2 : String = String::from("uint64");
 //let type2_2 : String = String::from("Address");
    
 //let description1_2 : Option<String> = Some("amount description".to_string());
 //let description2_2 : Option<String> = Some("account description".to_string());
 //let description3_2 : Option<String> = Some("misc description".to_string());
 
    
 //let signer = TransactionSigner::BasicAccount(acct1.clone());  
 
mod bar {
    use algonaut_abi::abi_interactions::AbiMethodArg;
    use algonaut_abi::abi_interactions::AbiReturn;
    use algonaut_abi::abi_type::AbiType;


    use algonaut_abi::abi_interactions::AbiMethod;

    pub struct Foo {
        pub name: String,
        pub description: String,
        pub type_: String, // still private
        pub parsed: Option<String>,
    }

    impl MyTrait for Foo {
        type Foo = Foo;
        type type_ = String;
        type parsed = Option<String>;

        fn new() -> Self::Foo {
            Foo {
                name: "".to_string(),
                description: "".to_string(),
                type_: "".to_string(),
                parsed: None,
            }
        }

        fn type_() -> String { "".to_string() }
        fn parsed() -> Option<AbiType> { None }
    }

    trait MyTrait {
        type Foo;
        type type_: ToString;
        type parsed;

        fn new() -> Self::Foo;
        fn type_() -> String;
        fn parsed() -> Option<AbiType>;
    }

    impl Foo {
        //Doc : https://developer.algorand.org/docs/get-details/transactions/signatures/#single-signatures
        //      https://developer.algorand.org/docs/get-details/dapps/smart-contracts/ABI/?from_query=Method%20Signature#reference-types
        //pub fn new() -> AbiMethod {
        //    let method_sig : String = "withdraw(uint64,account)void".to_string();
            //let method_sig : String = "add(uint64,uint64)uint128".to_string();

            
        //    println!("{}",&method_sig);

        //    AbiMethod::from_signature(&method_sig)
        //    .expect("Error")
            
        //}
        
        pub fn withdraw() -> AbiMethod {
            let method_sig : String = "withdraw(uint64,account)void".to_string();
            //let method_sig : String = "add(uint64,uint64)uint128".to_string();

            
            println!("{}",&method_sig);

            AbiMethod::from_signature(&method_sig)
            .expect("Error")
            
        }

        pub fn deposit() -> AbiMethod {
            let method_sig : String = "deposit(uint64,account)void".to_string();
            //let method_sig : String = "add(uint64,uint64)uint128".to_string();

            
            println!("{}",&method_sig);

            AbiMethod::from_signature(&method_sig)
            .expect("Error")
            
        }
   
    }
}



 let address_abi_type : AbiArgType = AbiArgType::AbiObj(Address);
   
  
  

    
 let _method : AbiMethod = bar::Foo::withdraw(); 

 let _method2 : AbiMethod = bar::Foo::deposit();
  

 //https://docs.rs/num-bigint/0.4.3/num_bigint/struct.BigUint.html
 let withdrw_amt : BigUint = BigUint::new(vec![0]); //in MicroAlgos
 
 //let withdrw_to_addr : BigUint = BigUint::new(vec![0]);
 //let addr_as_bytes: &[u8] = acct1.address().to_string().as_bytes();

 let mut withdrw_to_addr: [u8; 32] = [0; 32];

 //Converts Address to 32 Bit Bytes
 withdrw_to_addr.copy_from_slice(&acct1.address().to_string().as_bytes()[..32]);


 //withdrawal amount
 let arg1 : AbiArgValue = AbiArgValue::AbiValue( Int(withdrw_amt));
 
let arg2: AbiArgValue = AbiArgValue::AbiValue(algonaut_abi::abi_type::AbiValue::Address(OtherAddress::new(withdrw_to_addr)));


 const Q : usize = 0usize;

 let mut _hashmap: HashMap<usize, AbiMethod>= std::collections::HashMap::new(); //HashMap<usize, AbiMethod>




 
 _hashmap.insert(Q,_method2); 
 //_hashmap.insert(q,q);
    
 let _note : Option<Vec<u8>> = Some(vec![0]);

    
 println!("building Pay transaction");
 let t = TxnBuilder::with(

        &params2,

        Pay::new(acct1.address(), acct1.address(), MicroAlgos(123_456)).build(),

    )

    .build();
let t2 = t.unwrap().clone();

//let t3 = t2.clone();

//let sign_txn = acct1.sign_transaction(t2)?;

let _app_id = 161734720;

//let _Escrow_Address : Address = logic.get_application_address(_app_id);

 println!("building ABI Method Call transaction");

let mut atc = AtomicTransactionComposer::default();  

//Get Escrow Address From App ID, but no logic crate in Algonaut SDK

let Escrow_Address = to_app_address(_app_id.clone());

//Add method Call     
atc.add_method_call( &mut AddMethodCallParams {
                 app_id: _app_id,
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
).unwrap();


atc.build_group().expect("Error");

atc.execute(&algod).await.expect("Error");
   
   

println!("{:?}", &mut atc.status());


Ok(())
 
}

