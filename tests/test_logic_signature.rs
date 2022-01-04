use std::error::Error;

use algonaut_core::{CompiledTeal, LogicSignature, MultisigAddress, SignedLogic};

use algonaut_transaction::{account::Account, error::TransactionError};
use tokio::test;

// Reference:
// https://github.com/algorand/java-algorand-sdk/blob/840cf26043f475e43938c64fbda4526a874c258f/src/test/java/com/algorand/algosdk/crypto/TestLogicsigSignature.java

#[test]
async fn test_logic_sig_creation() -> Result<(), Box<dyn Error>> {
    let program = CompiledTeal(vec![
        0x01, 0x20, 0x01, 0x01, 0x22, // int 1
    ]);
    let args = vec![];
    let program_hash = "6Z3C3LDVWGMX23BMSYMANACQOSINPFIRF77H7N3AWJZYV6OH6GWTJKVMXY";
    let sender = program_hash.parse()?;

    let lsig = SignedLogic {
        logic: program.clone(),
        args,
        sig: LogicSignature::ContractAccount,
    };

    let verified = lsig.verify(sender);
    assert!(verified);
    assert_eq!(lsig.as_address(), sender);

    let args = vec![vec![1, 2, 3], vec![4, 5, 6]];
    let lsig = SignedLogic {
        logic: program.clone(),
        args,
        sig: LogicSignature::ContractAccount,
    };

    let verified = lsig.verify(sender);
    assert!(verified);
    assert_eq!(lsig.as_address(), sender);

    // serialization tests -> unit tests in api_model

    // check modified program fails on verification
    let mut modified_program = program;
    modified_program.0[3] = 0x03;

    let lsig = SignedLogic {
        logic: modified_program,
        args: vec![],
        sig: LogicSignature::ContractAccount,
    };

    assert!(!lsig.verify(sender));

    Ok(())
}

#[test]
async fn test_logic_sig_invalid_program_creation() {
    // TODO smart contract verification https://github.com/manuelmauro/algonaut/issues/25
    // Add an initializer to SignedLogic, return error if verification fails
}

#[test]
async fn test_logic_sig_signature() -> Result<(), Box<dyn Error>> {
    let program = CompiledTeal(vec![
        0x01, 0x20, 0x01, 0x01, 0x22, // int 1
    ]);
    let account = Account::generate();
    let sig = account.generate_program_sig(&program);

    let lsig = SignedLogic {
        logic: program.clone(),
        args: vec![],
        sig: LogicSignature::DelegatedSig(sig),
    };

    let verified = lsig.verify(account.address());
    assert!(verified);

    // serialization tests -> unit tests in api_model

    Ok(())
}

#[test]
async fn test_logic_sig_multisig_signature() -> Result<(), Box<dyn Error>> {
    let program = CompiledTeal(vec![
        0x01, 0x20, 0x01, 0x01, 0x22, // int 1
    ]);

    let one = "DN7MBMCL5JQ3PFUQS7TMX5AH4EEKOBJVDUF4TCV6WERATKFLQF4MQUPZTA".parse()?;
    let two = "BFRTECKTOOE7A5LHCF3TTEOH2A7BW46IYT2SX5VP6ANKEXHZYJY77SJTVM".parse()?;
    let three = "47YPQTIGQEO7T4Y4RWDYWEKV6RTR2UNBQXBABEEGM72ESWDQNCQ52OPASU".parse()?;

    let ma = MultisigAddress::new(1, 2, &vec![one, two, three])?;

    let acc1 = Account::from_mnemonic("auction inquiry lava second expand liberty glass involve ginger illness length room item discover ahead table doctor term tackle cement bonus profit right above catch")?;
    let acc2 = Account::from_mnemonic("since during average anxiety protect cherry club long lawsuit loan expand embark forum theory winter park twenty ball kangaroo cram burst board host ability left")?;
    let account = Account::generate();

    let msig = acc1.init_logic_msig(&program, &ma)?;

    let lsig = SignedLogic {
        logic: program.clone(),
        args: vec![],
        sig: LogicSignature::DelegatedMultiSig(msig.clone()),
    };
    let verified = lsig.verify(ma.address());
    assert!(!verified); // threshold not reached

    let msig_res = account.append_to_logic_msig(&program, msig.clone());
    assert!(msig_res.is_err());
    assert!(matches!(
        msig_res.as_ref().err().unwrap(),
        TransactionError::InvalidSecretKeyInMultisig
    ));

    let res = acc2.append_to_logic_msig(&program, msig);
    assert!(res.is_ok());

    let msig = res.unwrap();

    let lsig = SignedLogic {
        logic: program.clone(),
        args: vec![],
        sig: LogicSignature::DelegatedMultiSig(msig),
    };

    let verified = lsig.verify(ma.address());
    assert!(verified); // now threshold is reached

    // Add a single signature and ensure it fails
    // Remove and ensure it still works
    // tests omitted -> sig and msig can't be set at the same time here.

    // serialization tests -> unit tests in api_model

    Ok(())
}
