//code reference: https://github.com/manuelmauro/algonaut/blob/main/tests/step_defs/integration/abi.rs


  
 
mod bar {
    //use algonaut_abi::abi_interactions::AbiMethodArg;
    //use algonaut_abi::abi_interactions::AbiReturn;
    use algonaut_abi::abi_type::AbiType;


    use algonaut_abi::abi_interactions::AbiMethod;

    pub struct Foo {
        pub name: String,
        pub description: String,
        pub type_: String, 
        pub parsed: Option<String>,
    }

    impl MyTrait for Foo {
        type Foo = Foo;
        type Type = String;
        type Parsed = Option<String>;

        fn new() -> Self::Foo {
            Foo {
                name: "".to_string(),
                description: "".to_string(),
                type_: "".to_string(),
                parsed: None,
            }
        }

        fn r#type() -> String { "".to_string() }
        fn parsed() -> Option<AbiType> { None }
    }

    trait MyTrait {
        type Foo;
        type Type: ToString;
        type Parsed;

        fn new() -> Self::Foo;
        fn r#type() -> String;
        fn parsed() -> Option<AbiType>;
    }

    impl Foo {
        //Doc : https://developer.algorand.org/docs/get-details/transactions/signatures/#single-signatures
        //      https://developer.algorand.org/docs/get-details/dapps/smart-contracts/ABI/?from_query=Method%20Signature#reference-types
        // Boilerplate
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

            
            println!("Method Signature: {}",&method_sig);

            AbiMethod::from_signature(&method_sig)
            .expect("Error")
            
        }

        pub fn deposit() -> AbiMethod {
            let method_sig : String = "deposit(PaymentTransaction,account)void".to_string();
            //let method_sig : String = "add(uint64,uint64)uint128".to_string();

            
            println!("Method Signature: {}",&method_sig);

            AbiMethod::from_signature(&method_sig)
            .expect("Error")
            
        }
   
    }
}

//Custom Params Struct

mod params {


//use rmp_serde::from_slice;
pub mod ATC {
    /*
    Atomic Transaction Composer Required Traits
    */
    use std::string::String as str;

    pub enum AtomicTransactionComposerStatus {
        Building,
        Built,
        Signed,
        Submitted,
        Committed,
    }

    impl std::fmt::Display for AtomicTransactionComposerStatus {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                AtomicTransactionComposerStatus::Building => write!(f, "Building"),
                AtomicTransactionComposerStatus::Built => write!(f, "Built"),
                AtomicTransactionComposerStatus::Signed => write!(f, "Signed"),
                AtomicTransactionComposerStatus::Submitted => write!(f, "Submitted"),
                AtomicTransactionComposerStatus::Committed => write!(f, "Committed"),
            }
        }
    }

    pub trait Into {
        type Into;
        type From;
        type T;

        fn into<T: From<Self::T> + ?Sized>(_b: &T) {
            todo!()
        }

        fn into_boxed_str() -> &'static str;
    }

    // Implement the From trait for AtomicTransactionComposerStatus to &'static str
    impl<'a> From<AtomicTransactionComposerStatus> for &'a str {
        fn from(s: AtomicTransactionComposerStatus) -> &'a str {
            Box::leak(Box::new(s.to_string()))
        }
    }

    // Implement the From trait for &mut AtomicTransactionComposerStatus to &str
    impl<'a> From<&'a mut AtomicTransactionComposerStatus> for &'a str {
        fn from(_: &'a mut AtomicTransactionComposerStatus) -> &'a str {
            todo!()
        }
    }
}



pub mod params {
    use algonaut_core::SuggestedTransactionParams;
    //use std::collections::HashMap;
    //use crate::params::from_slice;
    use algonaut_model::algod::v2::TransactionParams;
    use algonaut_core::MicroAlgos;
    use algonaut_core::Round;
    //use algonaut_crypto::HashDigest;
    //use crate::params::SuggestedTransactionParams;

    pub struct MySuggestedTransactionParams(SuggestedTransactionParams);

        
        trait ToVariant {
            type Foo;
            type Params;
            type Parsed;
            type Payment;
            type MyTrait;
            

            fn _app_id(&self, x: u64) -> u64;
            fn default() -> Option<String>{ None }
            //fn parsed() -> Option<AbiType>;

            //fn suggested_tx_params(&self) -> OtherSuggestedTransactionParams { }

            fn to_variant(&self, params : SuggestedTransactionParams) -> TransactionParams { 
                let dict =  algonaut_model::algod::v2::TransactionParams{
                    consensus_version: params.consensus_version,
                    fee_per_byte: MicroAlgos(0u64),
                    genesis_hash: params.genesis_hash,//HashDigest([u8; 32]),
                    genesis_id: params.genesis_id,
                    last_round: Round(0u64),
                    min_fee: MicroAlgos(0u64),
                };

                dict
            }

        }

    }
}


//
pub mod escrow {

    use algonaut::algod::v2::Algod;
    use algonaut_abi::abi_type::AbiValue::Int;
    use algonaut_core::Address;
    
    use num_bigint::BigUint;

    use algonaut::{
        atomic_transaction_composer::{
            transaction_signer::TransactionSigner, AbiArgValue, //AbiMethodReturnValue,
            AtomicTransactionComposer, //AbiReturnDecodeError, AddMethodCallParams, 
            TransactionWithSigner, //AtomicTransactionComposerStatus, 
        },
        error::ServiceError,
    };
    use algonaut_abi::{
        abi_interactions::{AbiMethod}, //AbiArgType,AbiReturn,, ReferenceArgType,, AbiReturnType  
        //abi_type::{AbiType, AbiValue as OtherAbiValue},
    };
    use algonaut_core::{to_app_address, Address as OtherAddress, MicroAlgos, CompiledTeal};
    
    use algonaut_transaction::{
        builder::TxnFee, builder::TxnFee::Fixed,
        transaction::{ApplicationCallOnComplete, StateSchema},
        Pay, TxnBuilder,
    };

    use algonaut_core::SuggestedTransactionParams as OtherSuggestedTransactionParams;
    use algonaut_transaction::{transaction::Payment, account::Account};
  
    use algonaut_crypto::HashDigest;
  
    use std::convert::TryInto;
   
    //use crate::params::params::MySuggestedTransactionParams;
    
    #[derive(Debug)]
    //lifetime Parameter
    pub struct Foo <'a> {
        pub withdrw_amt: u32,
        pub withdrw_to_addr: [u8; 32],
        pub arg1: AbiArgValue,
        pub arg2: AbiArgValue,
        pub _app_id: u64,
        pub _escrow_address: Address,
        pub atc: &'a AtomicTransactionComposer,
    }

    trait MyTrait {
        type Foo <'a>;
        type Params;
        type Parsed;
        type Payment;

        fn _app_id(&self, x: u64) -> u64;
        //fn default() -> Option<String>{ None }
        //fn suggested_tx_params(&self) -> OtherSuggestedTransactionParams { OtherSuggestedTransactionParams::default() }
        fn arg1(withdrw_amt: u64) -> AbiArgValue { AbiArgValue::AbiValue(Int(withdrw_amt.into())) }
        fn arg2(withdrw_amt: u64) -> AbiArgValue { AbiArgValue::AbiValue(Int(withdrw_amt.into())) }
    }

    impl MyTrait for Foo <'_>{
        type Foo <'a> = Foo<'a>;
        type Parsed = Option<String>;
        type Payment = Option<Payment>;
        type Params = Option<OtherSuggestedTransactionParams>;
        fn _app_id(&self, x: u64) -> u64 { x }
    }

    impl Foo <'_> {
        // Adding method to create application call
        fn get_call(&self) -> Result<ApplicationCallOnComplete, ServiceError> {
            //let func_args = vec![self.arg1.clone(), self.arg2.clone()];
            
            todo!()
            
        }

        // Adding method to create pay transaction
        fn get_payment(&self) -> Result<Payment, ServiceError> {
            todo!()
           // tx
        }

        fn arg1(&self)-> AbiArgValue{ 
            todo!()
            
        }
        
        pub fn note(size : u32) -> Option <Vec<u8>>{
            Some(vec![size.try_into().unwrap()])

        }
    


        pub fn withdraw_amount(amount : u32) -> AbiArgValue {
            /*
            Converts a U64 int to Big Uint and returns an AbiArg Value
            
            */
            let withdrw_amt : BigUint = BigUint::new(vec![amount]); //in MicroAlgos
            

            let arg1 : AbiArgValue = AbiArgValue::AbiValue( Int(withdrw_amt));
            arg1


        }
            
    
        
        pub fn withdraw(_acct1: Account ){
             /* 
            Withdraw Method Parameters for Escrow SmartContract
            
                Docs: https://docs.rs/num-bigint/0.4.3/num_bigint/struct.BigUint.html

                Does nothing
            */

        }
        

        //use algonaut_core::Address;
        pub fn pay(to_address : algonaut_core::Address , acct1 : Account, _params : algonaut_core::SuggestedTransactionParams) -> algonaut_transaction::Transaction{
            /*
                Constructs a Payment Transaction to an Address
            */

             let _t = TxnBuilder::with(

                    &_params,

                    Pay::new(acct1.address(), to_address, MicroAlgos(123_456)).build(),

                )

                .build()
                .unwrap();
            
            return _t;
        }

        pub fn app_address (app_id : &u64) -> Address{
            to_app_address(*app_id)
        }
        
        pub fn deposit(_algod : Algod , acct1_3 : Account ,  params : algonaut_core::SuggestedTransactionParams) -> algonaut_core::SuggestedTransactionParams {
            /*
            Deposit Method Parameters for Escrow SmartContract
            Unused and Depreciated
           
            Does
            */

            //Params
            //
            //App ID
            let _app_id = 161737986;

            
            //Get Escrow Address From App ID

            let _escrow_address = Foo::app_address(&_app_id); //to_app_address(_app_id.clone());
           
            println!(" building Pay transaction to Escrow Address: {}", &_escrow_address);

            let _t = Foo::pay(_escrow_address, acct1_3.clone(), params.clone());                

            // create a transaction with signer with the current transaction

            let _signer = TransactionSigner::BasicAccount(acct1_3);


            let tx_with_signer = TransactionWithSigner { tx: _t, signer: _signer };


            let mut atc = AtomicTransactionComposer::default();  

            // Deposit
            // Add Payment Txn to 
            // Should Ideally Match To A Statemachine Behaviour Bloc
            atc.add_transaction(tx_with_signer).unwrap();

            params

 
        }

        pub fn new() -> AtomicTransactionComposer{
        /*
        Constructs a Default Atomic Transation Composer
        */
            AtomicTransactionComposer::default()
        
        }
     
        pub fn address_to_bytes(addr: String) -> [u8; 32]{ 
        /*
        Constructs a 32 Bit Byte Slice froma Given Address String
        */   
            let mut _to_addr: [u8; 32] = [0; 32];
            //_to_addr.copy_from_slice(&acct1.address().to_string().as_bytes()[..32]);
            _to_addr.copy_from_slice(&addr.as_bytes()[..32]);

            _to_addr
            
        }

        //let arg2: AbiArgValue = AbiArgValue::AbiValue(algonaut_abi::abi_type::AbiValue::Address(OtherAddress::new(withdrw_to_addr)));
      
        pub fn address(addr : [u8; 32]) -> AbiArgValue {
            /* Returns an Address abi value from an Address as [u8,32]*/
            AbiArgValue::AbiValue(algonaut_abi::abi_type::AbiValue::Address(OtherAddress::new(addr)))

        } 

        

        pub fn fee(amount : u64) -> TxnFee{Fixed(MicroAlgos(amount))}

        pub fn construct_app_call_method(
        /*
        Constructs an App Call Method as a Rust Module
        
        */
        
        //&AtomicTransactionComposer
        &self,
        //#[base] _base: &Node,
        _app_id: u64,
        _method: AbiMethod,
        _method_args: Vec<AbiArgValue>,
        _fee: TxnFee,//Fixed(MicroAlgos(2500u64)), //make customizable
        _sender: Address,
        _on_complete: ApplicationCallOnComplete,
        _clear_program: Option<CompiledTeal>,
        _global_schema: Option<StateSchema>,
        _local_schema: Option<StateSchema>,
        _extra_pages: u32,
        _note: Option<Vec<u8>>,
        _lease: Option<HashDigest>,
        _rekey_to: Option<Address>,
        _signer: TransactionSigner,
    
    
    ) -> Result<Foo<'_>, ServiceError> {
            todo!()
            
        }
        

    } 

}
    
use algonaut::atomic_transaction_composer::{ transaction_signer::TransactionSigner::BasicAccount};
use algonaut::transaction::{account::Account,
    transaction::ApplicationCallOnComplete::NoOp,
};
use algonaut::algod::v2::Algod;

use algonaut::{
    atomic_transaction_composer::{
        //transaction_signer::TransactionSigner, AbiArgValue, AbiMethodReturnValue,
        AddMethodCallParams, //AtomicTransactionComposer,AbiReturnDecodeError,
        AtomicTransactionComposerStatus, //TransactionWithSigner,
    },
    //error::ServiceError,
};
use algonaut_core::{to_app_address}; //Address as OtherAddress, , MicroAlgos, CompiledTeal
use std::error::Error;

use crate::escrow::Foo;
//use crate::params::ATC;

//#[macro_use]

#[tokio::main]



async fn main() -> Result<(), Box<dyn Error>> {


 
 
 let url = String::from("https://node.testnet.algoexplorerapi.io");
 
 let user = String::from("User-Agent");
 let pass = String::from("DoYouLoveMe?");
 let headers :  Vec<(&str, &str)> = vec![(&user, &pass)];

 let algod = Algod::with_headers(&url, headers)?;
 
 let acct1 = Account::from_mnemonic("degree feature waste gospel screen near subject boost wreck proof caution hen adapt fiber fault level blind entry also embark oval board bunker absorb garage")?;
 
 let acct1_2 = acct1.clone();
    
 //let acct1_3 = acct1.clone();
  

 println!("retrieving suggested params");
 let params = algod.suggested_transaction_params().await?;
 //let params2 = params.clone();
 
 
 let pages: u32 = 0;

 //let address_abi_type : AbiArgType = AbiArgType::AbiObj(Address);
   
  
  

 //Withdraw method   
 //let _withdraw : AbiMethod = bar::Foo::withdraw(); 

 //Deposit method
 // Buggy Method Signature
 //let _deposit : AbiMethod = bar::Foo::deposit();
  




 const Q : usize = 0usize;

 //let mut _hashmap: HashMap<usize, AbiMethod>= std::collections::HashMap::new(); //HashMap<usize, AbiMethod>




 
 //_hashmap.insert(Q,_deposit.clone()); 
    





    println!("building ABI Method Call transaction");

    let mut atc = escrow::Foo::new();  



    let mut _to_addr: [u8; 32] = Foo::address_to_bytes(acct1.address().to_string());//[0; 32];

    let __app_id : u64 = 161737986 ;

    //Txn Details As a Struct
    let details = escrow::Foo { 
            withdrw_amt : 0u32,//Foo::withdraw_amount(0u32),//BigUint::new(vec![0]),//BigUint { data: vec![0u64] },//BigUint = BigUint::new(vec![0]), 
            withdrw_to_addr: _to_addr.clone(), 
            arg1: Foo::withdraw_amount(0u32), 
            arg2: Foo::address(_to_addr),
            _app_id: __app_id.clone(), 
            _escrow_address: to_app_address(__app_id), 
            atc: &atc };

    println!("{:?}", &details);

            //Add method Call     
    atc.add_method_call( &mut AddMethodCallParams {
                    app_id: details._app_id,
                    method: bar::Foo::deposit(), //bar::Foo::withdraw() //for deposits
                    method_args: vec![details.arg1, details.arg2],
                    fee: Foo::fee(2500),
                    sender: acct1_2.address(),
                    suggested_params: params,
                    on_complete: NoOp,
                    approval_program: None,
                    clear_program: None,
                    global_schema: None,
                    local_schema: None,
                    extra_pages: pages,
                    note: Foo::note(0u32),//_note,
                    lease: None,
                    rekey_to: None,
                    signer: BasicAccount(acct1_2)
            
        }
    ).unwrap();


 atc.build_group().expect("Error");

 atc.execute(&algod).await.expect("Error");
   
 let status_str : &mut AtomicTransactionComposerStatus = &mut atc.status();

 //ATC::Into(status_str);
 //Buggy Trait Implementation
 //let t : _= <&mut algonaut::atomic_transaction_composer::AtomicTransactionComposerStatus as std::convert::Into<T>>::into(status_str);  // {
 
//       "BUILDING" => AtomicTransactionComposerStatus::Building,
 //       "BUILT" => AtomicTransactionComposerStatus::Built,
 //       "SIGNED" => AtomicTransactionComposerStatus::Signed,
 //       "SUBMITTED" => AtomicTransactionComposerStatus::Submitted,
 //       "COMMITTED" => AtomicTransactionComposerStatus::Committed,
 //       _ => panic!("Not handled status string: {:?}", status_str),
 //   };

 // Debugging Transaction Status
 println!("{:?}", status_str);



 // if status_str != atc.status() {
 //        panic!("status doesn't match");
 //    }

 Ok(())
 

}
