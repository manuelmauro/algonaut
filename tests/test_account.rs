use algonaut_core::{Address, CompiledTealBytes, LogicSignature, MicroAlgos, Round, SignedLogic};
use algonaut_core::{MultisigAddress, ToMsgPack};
use algonaut_crypto::HashDigest;
use algonaut_transaction::account::Account;
use algonaut_transaction::transaction::TransactionSignature;
use algonaut_transaction::{Pay, SignedTransaction, TxnBuilder};
use data_encoding::{BASE64, HEXLOWER};
use std::convert::TryInto;
use std::error::Error;
use tokio::test;

// Reference:
// https://github.com/algorand/java-algorand-sdk/blob/840cf26043f475e43938c64fbda4526a874c258f/src/test/java/com/algorand/algosdk/account/TestAccount.java

#[test]
async fn test_signs_transaction_e2e() -> Result<(), Box<dyn Error>> {
    // Note: Different reference strings than Java SDK. HashDigest can't be initialized with an empty array here.
    // This data was successfully tested with the Java SDK by modifying its test locally.
    let ref_sig_txn = "82a3736967c440844126d9a723338e54ecdeeed58e0a7c5a482d2007a255395e4dd7214784b0e54c9307238b12066d937874e965505e0b883fdde672b57fa5284ccc25c383960ca374786e88a3616d74cd04d2a3666565cd03e8a26676ce0001a04fa26768c4200101010101010101010101010101010101010101010101010101010101010101a26c76ce0001a437a3726376c4207d3f99e53d34ae49eb2f458761cf538408ffdaee35c70d8234166de7abe3e517a3736e64c4201bd63dc672b0bb29d42fcafa3422a4d385c0c8169bb01595babf8855cf596979a474797065a3706179";
    let ref_tx_id = "AJNRQXSGQONF7OEJRFC4ZIDRGZCGBAANRLXYHIA23DXMSBXQ3NBQ";

    let from_addr = "DPLD3RTSWC5STVBPZL5DIIVE2OC4BSAWTOYBLFN2X6EFLT2ZNF4SMX64UA".parse()?;
    let from_sk = "actress tongue harbor tray suspect odor load topple vocal avoid ignore apple lunch unknown tissue museum once switch captain place lemon sail outdoor absent creek";
    let to_addr = "PU7ZTZJ5GSXET2ZPIWDWDT2TQQEP7WXOGXDQ3ARUCZW6PK7D4ULSE6NYCE".parse()?;

    // build unsigned transaction
    let tx = TxnBuilder::new(
        MicroAlgos(1000),
        Round(106575),
        Round(107575),
        // non 0 array for easier comparison with (modified) Java SDK test. Java SDK doesn't serialize 0 value array.
        HashDigest([1; 32]),
        Pay::new(from_addr, to_addr, MicroAlgos(1234)).build(),
    )
    .build();

    let account = Account::from_mnemonic(from_sk)?;

    // public key test omitted: it's private and tested in Account
    // make sure address was correctly computed
    assert_eq!(account.address(), from_addr);

    // sign the transaction
    let signed_tx = account.sign_transaction(&tx)?;
    let signed_tx_bytes = signed_tx.to_msg_pack()?;
    let signed_tx_hex = HEXLOWER.encode(&signed_tx_bytes);
    assert_eq!(signed_tx_hex, ref_sig_txn);

    // verify transaction ID
    let tx_id = signed_tx.transaction_id;
    assert_eq!(tx_id, ref_tx_id);

    Ok(())
}

// Note: JavaSDK test significantly modified: Not passing 0 as first round.
// Can't use first round to test "zero value" as Option::None is the only valid zero value, and first round is mandatory.
// Passing "0" would cause the serializer to write "0" in the output, leading to a different result than the Java SDK.
// Optional fields are tested implicitly here by not being set in the transaction builder.
#[test]
async fn test_signs_transaction_zero_val_e2e() -> Result<(), Box<dyn Error>> {
    // Note: Different reference strings than Java SDK. HashDigest can't be initialized with an empty array here.
    // This data was successfully tested with the Java SDK by modifying its test locally.
    let ref_sig_txn = "82a3736967c440844126d9a723338e54ecdeeed58e0a7c5a482d2007a255395e4dd7214784b0e54c9307238b12066d937874e965505e0b883fdde672b57fa5284ccc25c383960ca374786e88a3616d74cd04d2a3666565cd03e8a26676ce0001a04fa26768c4200101010101010101010101010101010101010101010101010101010101010101a26c76ce0001a437a3726376c4207d3f99e53d34ae49eb2f458761cf538408ffdaee35c70d8234166de7abe3e517a3736e64c4201bd63dc672b0bb29d42fcafa3422a4d385c0c8169bb01595babf8855cf596979a474797065a3706179";
    let ref_tx_id = "AJNRQXSGQONF7OEJRFC4ZIDRGZCGBAANRLXYHIA23DXMSBXQ3NBQ";

    let from_addr = "DPLD3RTSWC5STVBPZL5DIIVE2OC4BSAWTOYBLFN2X6EFLT2ZNF4SMX64UA".parse()?;
    let from_sk = "actress tongue harbor tray suspect odor load topple vocal avoid ignore apple lunch unknown tissue museum once switch captain place lemon sail outdoor absent creek";
    let to_addr = "PU7ZTZJ5GSXET2ZPIWDWDT2TQQEP7WXOGXDQ3ARUCZW6PK7D4ULSE6NYCE".parse()?;

    // build unsigned transaction
    let tx = TxnBuilder::new(
        MicroAlgos(1000),
        // Java SDK 0 replaced with non zero value (see comment on test)
        Round(106575),
        Round(107575),
        HashDigest([1; 32]),
        Pay::new(from_addr, to_addr, MicroAlgos(1234)).build(),
    )
    .build();

    let account = Account::from_mnemonic(from_sk)?;

    // public key test omitted: it's private and tested in Account
    // make sure address was correctly computed
    assert_eq!(account.address(), from_addr);

    // sign the transaction
    let signed_tx = account.sign_transaction(&tx)?;
    let signed_tx_bytes = signed_tx.to_msg_pack()?;
    let signed_tx_hex = HEXLOWER.encode(&signed_tx_bytes);
    assert_eq!(signed_tx_hex, ref_sig_txn);

    // verify transaction ID
    let tx_id = signed_tx.transaction_id;
    assert_eq!(tx_id, ref_tx_id);

    Ok(())
}

// testKeygen() -> unit test in Account
// testToMnemonic() -> unit test in Account

#[test]
async fn test_sign_multisig_transaction() -> Result<(), Box<dyn Error>> {
    let addr = make_test_msig_addr()?;

    // build unsigned transaction
    let tx = TxnBuilder::new(
        MicroAlgos(217000),
        Round(972508),
        Round(973508),
        // non 0 array for easier comparison with (modified) Java SDK test. Java SDK doesn't serialize 0 value array.
        HashDigest([1; 32]),
        Pay::new(
            addr.address(),
            "DN7MBMCL5JQ3PFUQS7TMX5AH4EEKOBJVDUF4TCV6WERATKFLQF4MQUPZTA".parse()?,
            MicroAlgos(5000),
        )
        .build(),
    )
    .genesis_id("testnet-v31.0".to_owned())
    .note(BASE64.decode(b"tFF5Ofz60nE=")?)
    .build();

    let account = Account::from_mnemonic("auction inquiry lava second expand liberty glass involve ginger illness length room item discover ahead table doctor term tackle cement bonus profit right above catch")?;

    let msig = account.init_transaction_msig(&tx, &addr)?;
    let signed_tx = SignedTransaction {
        transaction: tx,
        transaction_id: "".to_owned(),
        sig: TransactionSignature::Multi(msig),
    };

    let enc = rmp_serde::to_vec_named(&signed_tx)?;

    // check the bytes convenience function is correct
    assert_eq!(signed_tx.to_msg_pack()?, enc);

    // check main signature is correct
    // Note: Different reference strings than Java SDK. HashDigest can't be initialized with an empty array here.
    // This data was successfully tested with the Java SDK by modifying its test locally.
    let golden = BASE64.decode(b"gqRtc2lng6ZzdWJzaWeTgqJwa8QgG37AsEvqYbeWkJfmy/QH4QinBTUdC8mKvrEiCairgXihc8RAMV11j0KD/w9j2Ni7Xqd/ETAl6c2+PgdoOqy0laOIIXeJfbMo03eOn4LuMmc9PkqZCfdyb8b9FYkD5ZBpE9QpBoGicGvEIAljMglTc4nwdWcRdzmRx9A+G3PIxPUr9q/wGqJc+cJxgaJwa8Qg5/D4TQaBHfnzHI2HixFV9GcdUaGFwgCQhmf0SVhwaKGjdGhyAqF2AaN0eG6Ko2FtdM0TiKNmZWXOAANPqKJmds4ADtbco2dlbq10ZXN0bmV0LXYzMS4womdoxCABAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAaJsds4ADtrEpG5vdGXECLRReTn8+tJxo3JjdsQgG37AsEvqYbeWkJfmy/QH4QinBTUdC8mKvrEiCairgXijc25kxCCNkrSJkAFzoE36Q1mjZmpq/OosQqBd2cH3PuulR4A36aR0eXBlo3BheQ==")?;
    assert_eq!(enc, golden);

    Ok(())
}

#[test]
#[ignore]
async fn test_append_multisig_transaction() -> Result<(), Box<dyn Error>> {
    // TODO Account function to append signature to raw transaction
    Ok(())
}

// Java SDK test significantly changed: starting with transaction instead of signed transaction,
// in Java the signed transaction signatures are empty. It's not possible to create a signed transaction with this state.
// COMMENTED: TODO deserialization fixes/checks https://github.com/manuelmauro/algonaut/issues/96, https://github.com/manuelmauro/algonaut/issues/84
// Also, this kind of integration test might not be necessary: https://github.com/manuelmauro/algonaut/issues/67 + unit tests for multisig might be enough
// #[test]
// async fn test_sign_multisig_key_reg_transaction() -> Result<(), Box<dyn Error>> {
//     let addr = make_test_msig_addr()?;
//     let enc_key_reg_tx = BASE64.decode(b"jKNmZWUAomZ2zQU/o2dlbqh0bjUwZS12MaJnaMQgb8FCXLbTzbIQLtZBNGJg6vNd8Uzvghi5wKZgnnnZWwiibHbNCSemc2Vsa2V5xCADez7ZuAqVsb2ohoDjAmyusmXyZUobNOn+HqAYTJmYCKNzbmTEII2StImQAXOgTfpDWaNmamr86ixCoF3Zwfc+66VHgDfppHR5cGWma2V5cmVnp3ZvdGVmc3TNBT+mdm90ZWtkzScQp3ZvdGVrZXnEICoC+altY7RwEG9ZUDSCrqhwag1l0Zm+xk5gTfdmv7Dkp3ZvdGVsc3TOAC3L/w==")?;

//     let tx: Transaction = rmp_serde::from_slice(&enc_key_reg_tx)?;

//     let account = Account::from_mnemonic("auction inquiry lava second expand liberty glass involve ginger illness length room item discover ahead table doctor term tackle cement bonus profit right above catch")?;

//     let stx = account.sign_multisig_transaction(&addr, &tx)?;
//     let enc = rmp_serde::to_vec_named(&stx)?;

//     // check signature is correct
//     let golden = BASE64.decode(b"gqRtc2lng6ZzdWJzaWeTgqJwa8QgG37AsEvqYbeWkJfmy/QH4QinBTUdC8mKvrEiCairgXihc8RAG3X483SK2WWO+HofVkDqCSReuFOc03jRKSVSVhXPcSVFUPBG8NErbFsy1/mSJIybxTJ14KQUy81GBLHL4O7+DoGicGvEIAljMglTc4nwdWcRdzmRx9A+G3PIxPUr9q/wGqJc+cJxgaJwa8Qg5/D4TQaBHfnzHI2HixFV9GcdUaGFwgCQhmf0SVhwaKGjdGhyAqF2AaN0eG6Mo2ZlZQCiZnbNBT+jZ2VuqHRuNTBlLXYxomdoxCBvwUJcttPNshAu1kE0YmDq813xTO+CGLnApmCeedlbCKJsds0JJ6ZzZWxrZXnEIAN7Ptm4CpWxvaiGgOMCbK6yZfJlShs06f4eoBhMmZgIo3NuZMQgjZK0iZABc6BN+kNZo2ZqavzqLEKgXdnB9z7rpUeAN+mkdHlwZaZrZXlyZWendm90ZWZzdM0FP6Z2b3Rla2TNJxCndm90ZWtlecQgKgL5qW1jtHAQb1lQNIKuqHBqDWXRmb7GTmBN92a/sOSndm90ZWxzdM4ALcv/")?;
//     assert_eq!(enc, golden);

//     Ok(())
// }

// #[test]
// COMMENTED: TODO deserialization fixes/checks https://github.com/manuelmauro/algonaut/issues/96, https://github.com/manuelmauro/algonaut/issues/84
// Also, this kind of integration test might not be necessary: https://github.com/manuelmauro/algonaut/issues/67 + unit tests for multisig might be enough
// async fn test_append_multisig_key_reg_transaction() -> Result<(), Box<dyn Error>> {
//     let addr = make_test_msig_addr()?;
//     // The base64 str from the Java SDK was replaced because it represents an invalid signed transaction (empty signatures and no genesis hash) and this fails deserialization.
//     let enc_key_reg_tx = BASE64.decode(b"gqRtc2lng6ZzdWJzaWeTgaJwa8QgG37AsEvqYbeWkJfmy/QH4QinBTUdC8mKvrEiCairgXiConBrxCAJYzIJU3OJ8HVnEXc5kcfQPhtzyMT1K/av8BqiXPnCcaFzxECWshb+R63FX/1/LfgKIzih2OQGy17nM3GsljvoTPEBReCO7y5i99yr1h76U6z61YE3UIh2yvCg8fALYhkxbPcFgaJwa8Qg5/D4TQaBHfnzHI2HixFV9GcdUaGFwgCQhmf0SVhwaKGjdGhyAqF2AaN0eG6Mo2ZlZQCiZnYco2dlbqh0bjUwZS12MaJnaMQgb8FCXLbTzbIQLtZBNGJg6vNd8Uzvghi5wKZgnnnZWwiibHbNBASmc2Vsa2V5xCADez7ZuAqVsb2ohoDjAmyusmXyZUobNOn+HqAYTJmYCKNzbmTEII2StImQAXOgTfpDWaNmamr86ixCoF3Zwfc+66VHgDfppHR5cGWma2V5cmVnp3ZvdGVmc3QcpnZvdGVrZM0nEKd2b3Rla2V5xCAqAvmpbWO0cBBvWVA0gq6ocGoNZdGZvsZOYE33Zr+w5Kd2b3RlbHN0zgAtxtw=")?;

//     let wrapped_tx: SignedTransaction = rmp_serde::from_slice(&enc_key_reg_tx)?;

//     let account = Account::from_mnemonic("auction inquiry lava second expand liberty glass involve ginger illness length room item discover ahead table doctor term tackle cement bonus profit right above catch")?;

//     let stx = account.sign_multisig_transaction(&addr, &wrapped_tx.transaction)?;
//     let enc = rmp_serde::to_vec_named(&stx)?;

//     // check signature is correct
//     let golden = BASE64.decode(b"gqRtc2lng6ZzdWJzaWeTgqJwa8QgG37AsEvqYbeWkJfmy/QH4QinBTUdC8mKvrEiCairgXihc8RAoAi0Wpdp3KZKz73LEn5t4UbYVZqcIv1TmZypvIshchGXt3JMpNX9b6SbKWG/ei/IOGWqkKAMsoxAUn+3/vcOAoGicGvEIAljMglTc4nwdWcRdzmRx9A+G3PIxPUr9q/wGqJc+cJxgaJwa8Qg5/D4TQaBHfnzHI2HixFV9GcdUaGFwgCQhmf0SVhwaKGjdGhyAqF2AaN0eG6Mo2ZlZQCiZnYco2dlbqh0bjUwZS12MaJnaMQgb8FCXLbTzbIQLtZBNGJg6vNd8Uzvghi5wKZgnnnZWwiibHbNBASmc2Vsa2V5xCADez7ZuAqVsb2ohoDjAmyusmXyZUobNOn+HqAYTJmYCKNzbmTEII2StImQAXOgTfpDWaNmamr86ixCoF3Zwfc+66VHgDfppHR5cGWma2V5cmVnp3ZvdGVmc3QcpnZvdGVrZM0nEKd2b3Rla2V5xCAqAvmpbWO0cBBvWVA0gq6ocGoNZdGZvsZOYE33Zr+w5Kd2b3RlbHN0zgAtxtw=")?;
//     assert_eq!(enc, golden);

//     Ok(())
// }

#[test]
#[ignore]
async fn merge_multisig_transaction_bytes() -> Result<(), Box<dyn Error>> {
    // TODO Account function to merge multisig txn bytes (used for raw transaction files)
    // (ideally don't port mergeMultisigTransactions: it was intentionally removed)
    Ok(())
}

// testSignBytes() -> unit test in Account
// testVerifyBytes() -> unit test in Account

#[test]
async fn test_logic_sig_transaction() -> Result<(), Box<dyn Error>> {
    let from: Address = "47YPQTIGQEO7T4Y4RWDYWEKV6RTR2UNBQXBABEEGM72ESWDQNCQ52OPASU".parse()?;
    let to: Address = "PNWOET7LLOWMBMLE4KOCELCX6X3D3Q4H2Q4QJASYIEOF7YIPPQBG3YQ5YI".parse()?;
    let account = Account::from_mnemonic("advice pudding treat near rule blouse same whisper inner electric quit surface sunny dismiss leader blood seat clown cost exist hospital century reform able sponsor")?;

    // build unsigned transaction
    let tx = TxnBuilder::new(
        MicroAlgos(1000),
        Round(2063137),
        Round(2063137 + 1000),
        HashDigest(
            BASE64
                // Note: JavaSDK's string modified to have valid base64 length (appended "=")
                .decode(b"sC3P7e2SdbqKJK0tbiCdK9tdSpbe6XeCGKdoNzmlj0E=")?
                .try_into()
                .unwrap(),
        ),
        Pay::new(from, to, MicroAlgos(2000)).build(),
    )
    .genesis_id("devnet-v1.0".to_owned())
    .note(BASE64.decode(b"8xMCTuLQ810=")?)
    .build();

    let program = CompiledTealBytes(vec![
        0x01, 0x20, 0x01, 0x01, 0x22, // int 1
    ]);
    let args = vec![vec![49, 50, 51], vec![52, 53, 54]];
    let signature = account.generate_program_sig(&program);

    // TODO move this to Account and verify against sender address
    let signed_t = SignedTransaction {
        transaction: tx.clone(),
        transaction_id: tx.id()?.to_owned(),
        sig: TransactionSignature::Logic(SignedLogic {
            logic: program,
            args,
            sig: LogicSignature::DelegatedSig(signature),
        }),
    };

    let golden_tx = "gqRsc2lng6NhcmeSxAMxMjPEAzQ1NqFsxAUBIAEBIqNzaWfEQE6HXaI5K0lcq50o/y3bWOYsyw9TLi/oorZB4xaNdn1Z14351u2f6JTON478fl+JhIP4HNRRAIh/I8EWXBPpJQ2jdHhuiqNhbXTNB9CjZmVlzQPoomZ2zgAfeyGjZ2Vuq2Rldm5ldC12MS4womdoxCCwLc/t7ZJ1uookrS1uIJ0r211Klt7pd4IYp2g3OaWPQaJsds4AH38JpG5vdGXECPMTAk7i0PNdo3JjdsQge2ziT+tbrMCxZOKcIixX9fY9w4fUOQSCWEEcX+EPfAKjc25kxCDn8PhNBoEd+fMcjYeLEVX0Zx1RoYXCAJCGZ/RJWHBooaR0eXBlo3BheQ==";
    assert_eq!(
        BASE64.encode(&rmp_serde::to_vec_named(&signed_t)?),
        golden_tx
    );

    Ok(())
}

// testTealSign() -> unit test in Account
// testToSeed() -> unit test in Account

fn make_test_msig_addr() -> Result<MultisigAddress, Box<dyn Error>> {
    MultisigAddress::new(
        1,
        2,
        &[
            "DN7MBMCL5JQ3PFUQS7TMX5AH4EEKOBJVDUF4TCV6WERATKFLQF4MQUPZTA".parse()?,
            "BFRTECKTOOE7A5LHCF3TTEOH2A7BW46IYT2SX5VP6ANKEXHZYJY77SJTVM".parse()?,
            "47YPQTIGQEO7T4Y4RWDYWEKV6RTR2UNBQXBABEEGM72ESWDQNCQ52OPASU".parse()?,
        ],
    )
    .map_err(|s| s.into())
}
