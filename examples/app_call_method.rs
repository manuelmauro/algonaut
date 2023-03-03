//code reference: https://github.com/manuelmauro/algonaut/blob/main/tests/step_defs/integration/abi.rs


  
 
mod bar {
    use algonaut_abi::abi_interactions::AbiMethodArg;
    use algonaut_abi::abi_interactions::AbiReturn;
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

//#[derive(Deref, DerefMut, From)]
mod params {

use algonaut_core::SuggestedTransactionParams;
use algonaut_core::MicroAlgos;
use algonaut_core::Round;
use algonaut_crypto::HashDigest;
use std::collections::HashMap; //inplace of dictionary crate
use core::fmt::Error;


use rmp_serde::from_slice;
pub mod atc {
    /*
    Atomic Transaction Composer Required Traits
    */
    use algonaut::atomic_transaction_composer::AtomicTransactionComposerStatus as OtherAtomicTransactionComposerStatus;
    use std::string::String as str;
    use algonaut::atomic_transaction_composer::AtomicTransactionComposerStatus::Building;


    pub enum AtomicTransactionComposerStatus {
        Building,
        Built,
        Signed,
        Submitted,
        Committed,
    }


    trait MyTrait {
        type Into: 'static;
    }

    // Implement the trait for AtomicTransactionComposerStatus
    impl MyTrait for AtomicTransactionComposerStatus {
        type Into = &'static str;
    }

    
    // Implement the Into trait for &'a AtomicTransactionComposerStatus to &'a str
    impl<'a> Into<&'a str> for &'a AtomicTransactionComposerStatus {
        fn into(self) -> &'a str {
            (&*self).into()
        }
    }
    // Implement the From trait for AtomicTransactionComposerStatus to &'static str
    impl <'a> From<AtomicTransactionComposerStatus> for &'a str {
        //fn from(_: AtomicTransactionComposerStatus) -> Self { todo!() }
 
        fn from(_: AtomicTransactionComposerStatus) -> &'a str  { todo!() }
 

        //fn from(status: AtomicTransactionComposerStatus) -> &'a str {
            
        //    match status {
        //        AtomicTransactionComposerStatus::Building => &String::from("Building"),//&Building,
        //        AtomicTransactionComposerStatus::Built => &String::from("Built"),
        //        AtomicTransactionComposerStatus::Signed => &String::from("Signed"),
        //        AtomicTransactionComposerStatus::Submitted => &String::from("Submitted"),
        //        AtomicTransactionComposerStatus::Committed => &String::from("Committed"),
        //        _ => panic!()
        //    }
        //}
    }

}


pub mod params {
    use algonaut_core::SuggestedTransactionParams;
    use std::collections::HashMap;
    use crate::params::from_slice;
    use algonaut_model::algod::v2::TransactionParams;
    use algonaut_core::MicroAlgos;
    use algonaut_core::Round;
    use algonaut_crypto::HashDigest;
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

            fn to_variant(&self, Params : SuggestedTransactionParams) -> TransactionParams { 
                let dict =  algonaut_model::algod::v2::TransactionParams{
                    consensus_version: Params.consensus_version,
                    fee_per_byte: MicroAlgos(0u64),
                    genesis_hash: Params.genesis_hash,//HashDigest([u8; 32]),
                    genesis_id: Params.genesis_id,
                    last_round: Round(0u64),
                    min_fee: MicroAlgos(0u64),
                };

                dict
            }

            //fn from_variant(variant: &Self) -> Result<Self, Error>
            
        

        //used when constructing To a variant
        //impl ToVariant for MySuggestedTransactionParams {
           
        //    fn to_variant(&self) -> SuggestedTransactionParams {
        //        let dict = TrasactionParams::new();

                
        //        dict
        //    }
        //}

    // impl FromVariant for MySuggestedTransactionParams {
    //     fn from_variant(variant: &Self) -> Result<Self, Error> {
    //         let dict = variant
    //             .to::<HashMap>()
    //             .unwrap();
                    //.ok_or(FromVariantError::InvalidVariantType {
                    //    variant_type: variant.get_type(),
                    //    expected: VariantType::Dictionary,
                    //})?;

    //         let t = SuggestedTransactionParams {
    //             genesis_id: "".to_string(),
    //             first_valid: Round(0u64),
    //             last_valid: Round(0u64),
    //             consensus_version: "".to_string(),
    //             min_fee: MicroAlgos(0u64),
    //             fee_per_byte: MicroAlgos(0u64),
    //             genesis_hash: HashDigest,
    //         };
    //         Ok()
    //     }
        }

    }
}


pub mod escrow {

    use algonaut::algod::v2::Algod;
    use algonaut_abi::abi_type::AbiValue::Int;
    use algonaut_core::Address;
    
    use num_bigint::BigUint;

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

    use algonaut_core::SuggestedTransactionParams as OtherSuggestedTransactionParams;
    use algonaut_transaction::transaction::Payment;

    //combine both
    use algonaut_transaction::account::Account;
    use algonaut_transaction::Transaction;

    use algonaut_crypto::HashDigest;
    use algonaut_core::Round;
    //use algonaut_core::SuggestedTransactionParams as OtherSuggestedTransactionParams;

    //my cyustom params
    use crate::params::params::MySuggestedTransactionParams;
    
    //lifetime Parameter
    pub struct Foo <'a> {
        pub withdrw_amt: BigUint,
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
            let func_args = vec![self.arg1.clone(), self.arg2.clone()];
            //let arg_types = vec![
            //    AbiArgType::Uint(64),
            //    AbiArgType::Uint(64)
            //];
            //let app_args = AbiMethod::encode_args(func_args, arg_types)?;

            let app_args = todo!();
            //app_args
            //let app_call = ApplicationCallOnComplete::new(self._app_id, "some_function_name", Some(app_args), None, None)?;
            //Ok(app_call)
        }

        // Adding method to create pay transaction
        fn get_payment(&self) -> Result<Payment, ServiceError> {
            let tx = todo!();
           // tx
        }

        fn arg1(&self)-> AbiArgValue{ 
            let x = todo!();
            //x
        }
        
    
    

    
        
        pub fn withdraw(acct1: Account ){
             /* 
            Withdraw Method Parameters for Escrow SmartContract
            
                Docs: https://docs.rs/num-bigint/0.4.3/num_bigint/struct.BigUint.html

            
            */

            let withdrw_amt : BigUint = BigUint::new(vec![0]); //in MicroAlgos
            
            // Address As Bytes
            let mut withdrw_to_addr: [u8; 32] = [0; 32];

            //Converts Address to 32 Bit Bytes
            //Should ideally be a method
            withdrw_to_addr.copy_from_slice(&acct1.address().to_string().as_bytes()[..32]);


            //App Call Arguments For AtomicTransactioComposer
            // Withdraw Method
            let arg1 : AbiArgValue = AbiArgValue::AbiValue( Int(withdrw_amt));
            
            let arg2: AbiArgValue = AbiArgValue::AbiValue(algonaut_abi::abi_type::AbiValue::Address(OtherAddress::new(withdrw_to_addr)));
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

        
        pub fn deposit(algod : Algod , acct1_3 : Account ,  params : algonaut_core::SuggestedTransactionParams) -> algonaut_core::SuggestedTransactionParams {
            /*
            Deposit Method Parameters for Escrow SmartContract
            */

            //Params
            //let params = algod.suggested_transaction_params();

            //App ID
            let _app_id = 161737986;

            
            //Get Escrow Address From App ID

            let _escrow_address = to_app_address(_app_id.clone());
           
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

     

    } 

}
    
use algonaut::atomic_transaction_composer::{ transaction_signer::TransactionSigner::BasicAccount};
use algonaut::transaction::{account::Account,
    transaction::ApplicationCallOnComplete::NoOp,
};
use algonaut_transaction::builder::TxnFee::Fixed;

use algonaut_abi::abi_interactions::AbiReturnType::Void;


use algonaut::atomic_transaction_composer::{ AbiArgValue::AbiValue};

use algonaut_abi::abi_interactions::{AbiMethodArg};
use num_bigint::BigUint;
use algonaut_abi::abi_type::AbiType::Address;
use algonaut_abi::abi_type::AbiValue::Int;


use algonaut_crypto::HashDigest;
use algonaut::algod::v2::Algod;


use algonaut::atomic_transaction_composer::AtomicTransactionComposerStatus::Building;

use std::collections::HashMap;


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

use crate::escrow::Foo;

#[macro_use]

#[tokio::main]

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
 //let params2 = params.clone();
 
 
 let pages: u32 = 0;

 let address_abi_type : AbiArgType = AbiArgType::AbiObj(Address);
   
  
  

 //Withdraw method   
 let _withdraw : AbiMethod = bar::Foo::withdraw(); 

 //Deposit method
 // Buggy Method Signature
 //let _deposit : AbiMethod = bar::Foo::deposit();
  




 const Q : usize = 0usize;

 //let mut _hashmap: HashMap<usize, AbiMethod>= std::collections::HashMap::new(); //HashMap<usize, AbiMethod>




 
 //_hashmap.insert(Q,_deposit.clone()); 
    
 let _note : Option<Vec<u8>> = Some(vec![0]);






    println!("building ABI Method Call transaction");

    let mut atc = AtomicTransactionComposer::default();  

    let mut _to_addr: [u8; 32] = [0; 32];

    _to_addr.copy_from_slice(&acct1.address().to_string().as_bytes()[..32]);

    //let arg1 : AbiArgValue = AbiArgValue::AbiValue( Int(withdrw_amt));
            
    //let arg2: AbiArgValue = AbiArgValue::AbiValue(algonaut_abi::abi_type::AbiValue::Address(OtherAddress::new(withdrw_to_addr)));
       

    //Txn Details As a Struct
    let details = escrow::Foo { 
            withdrw_amt : BigUint::new(vec![0]),//BigUint { data: vec![0u64] },//BigUint = BigUint::new(vec![0]), 
            withdrw_to_addr: _to_addr.clone(), 
            arg1: AbiArgValue::AbiValue( Int(0u64.into())), 
            arg2: AbiArgValue::AbiValue(algonaut_abi::abi_type::AbiValue::Address(OtherAddress::new(_to_addr))), 
            _app_id: 161737986, 
            _escrow_address: to_app_address(161737986), 
            atc: &atc };

    let q = details.arg1;

    let p = details.arg2;
    //Add method Call     
    atc.add_method_call( &mut AddMethodCallParams {
                    app_id: 161737986,//escrow::Foo::_app_id,
                    method: _withdraw,
                    method_args: vec![q, p],
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
   
 let status_str : &mut AtomicTransactionComposerStatus = &mut atc.status();

// let t = match status_str.into() {
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
