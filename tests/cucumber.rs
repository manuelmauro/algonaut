use data_encoding::BASE64;

use algosdk::account::Account;
use algosdk::auction::{Bid, SignedBid};
use algosdk::algod::models::NodeStatus;
use algosdk::AlgodClient;
use algosdk::KmdClient;
use algosdk::crypto::{Address, MultisigAddress};
use algosdk::transaction::{SignedTransaction, Transaction};
use algosdk::{
    mnemonic, Ed25519PublicKey, HashDigest, MasterDerivationKey, MicroAlgos, Round, VotePK, VRFPK,
};
use cucumber::{cucumber, Steps, StepsBuilder};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::time::Duration;

#[derive(Default)]
pub struct World {
    algod_client: Option<AlgodClient>,
    kmd_client: Option<KmdClient>,
    versions: Vec<String>,
    status: Option<NodeStatus>,
    status_after: Option<NodeStatus>,
    transaction_id: Option<String>,
    last_round: Option<Round>,
    params_fee: Option<MicroAlgos>,
    wallet_name: Option<String>,
    wallet_password: Option<String>,
    wallet_id: Option<String>,
    wallet_handle: Option<String>,
    public_keys: Vec<Ed25519PublicKey>,
    account: Option<Account>,
    accounts: Vec<String>,
    old_address: Option<String>,
    address: Option<String>,
    public_key: Option<Address>,
    mdk: Option<[u8; 32]>,
    new_mnemonic: Option<String>,
    fee: Option<MicroAlgos>,
    first_valid: Option<Round>,
    last_valid: Option<Round>,
    note: Vec<u8>,
    genesis_hash: Option<HashDigest>,
    genesis_id: String,
    vote_pk: Option<VotePK>,
    vrf_pk: Option<VRFPK>,
    vote_first: Option<Round>,
    vote_last: Option<Round>,
    vote_key_dilution: Option<u64>,
    receiver: Option<Address>,
    close: Option<Address>,
    amount: Option<MicroAlgos>,
    multisig: Option<MultisigAddress>,
    transaction: Option<Transaction>,
    signed_transaction: Option<SignedTransaction>,
    signed_transaction_bytes: Vec<u8>,
    signed_transactions: Option<Vec<SignedTransaction>>,
    micro_algos: Option<MicroAlgos>,
    bid: Option<Bid>,
    old_bid: Option<SignedBid>,
    signed_bid: Option<SignedBid>,
    err: bool,
    num: String,
}

impl cucumber::World for World {}

pub fn steps() -> Steps<World> {
    let mut builder: StepsBuilder<World> = Default::default();
    builder
        .given("an algod client", |world: &mut World, _step| {
            let home = dirs::home_dir().expect("Couldn't get home dir");
            let data_dir = home.join("node/network/Node");
            let token = {
                let f = File::open(data_dir.join("algod.token")).expect("Couldn't open algod.token");
                let mut f = BufReader::new(f);
                let mut token = String::new();
                f.read_line(&mut token).unwrap();
                token
            };
            let address = {
                let f = File::open(data_dir.join("algod.net")).expect("Couldn't open algod.net");
                let mut f = BufReader::new(f);
                let mut address = "http://".to_string();
                f.read_line(&mut address).unwrap();
                address
            };
            world.algod_client = Some(AlgodClient::new(&address, &token))
        })
        .given("a kmd client", |world: &mut World, _step| {
            let home = dirs::home_dir().expect("Couldn't get home dir");
            let kmd_dir = std::env::var("KMD_DIR").expect("KMD_DIR is not set");
            let data_dir = home.join("node/network/Node").join(&kmd_dir);
            let token = {
                let f = File::open(data_dir.join("kmd.token")).expect("Couldn't open kmd.token");
                let mut f = BufReader::new(f);
                let mut token = String::new();
                f.read_line(&mut token).unwrap();
                token
            };
            let address = {
                let f = File::open(data_dir.join("kmd.net")).expect("Couldn't open kmd.net");
                let mut f = BufReader::new(f);
                let mut address = "http://".to_string();
                f.read_line(&mut address).unwrap();
                address
            };
            world.kmd_client = Some(KmdClient::new(&address, &token))
        })
        .given("wallet information", |world: &mut World, _step| {
            //FIXME Sometimes this doesn't get created
            let wallet_name = "unencrypted-default-wallet";
            let wallet_password = "";
            let kmd_client = world.kmd_client.as_ref().unwrap();
            for wallet in kmd_client.list_wallets().unwrap().wallets {
                if wallet.name == wallet_name {
                    world.wallet_id = Some(wallet.id);
                }
            }
            let wallet_id = world.wallet_id.as_ref().expect("Could not find wallet unencrypted-default-wallet");
            let handle = kmd_client.init_wallet_handle(wallet_id, wallet_password).unwrap().wallet_handle_token;
            world.accounts = kmd_client.list_keys(&handle).unwrap().addresses;
            world.wallet_handle = Some(handle);
            world.wallet_name = Some(wallet_name.to_string());
            world.wallet_password = Some(wallet_password.to_string())
        })
        .then("the node should be healthy", |world: &mut World, _step| {
            let algod_client = world.algod_client.as_ref().unwrap();
            algod_client.health().unwrap();
        })
        .then("I get the ledger supply", |world: &mut World, _step| {
            let algod_client = world.algod_client.as_ref().unwrap();
            let _ = algod_client.ledger_supply().unwrap();
        })
        .then("I get transactions by address and round", |world: &mut World, _step| {
            let algod_client = world.algod_client.as_ref().unwrap();
            let _ = algod_client.transactions(&world.accounts[0], Some(Round(1)), Some(algod_client.status().unwrap().last_round), None, None, Some(10)).unwrap();
        })
        .then("I get transactions by address only", |world: &mut World, _step| {
            let algod_client = world.algod_client.as_ref().unwrap();
            let _ = algod_client.transactions(&world.accounts[0], None, None, None, None, Some(10)).unwrap();
        })
        .then("I get transactions by address and date", |world: &mut World, _step| {
            let algod_client = world.algod_client.as_ref().unwrap();
            let _ = algod_client.transactions(&world.accounts[0], None, None, Some(chrono::Local::today().naive_local().to_string()), Some(chrono::Local::today().naive_local().to_string()), Some(10)).unwrap();
        })
        .when_regex(r"^I get recent transactions, limited by (\d+) transactions$", |world: &mut World, strings, _step| {
            let algod_client = world.algod_client.as_ref().unwrap();
            let _ = algod_client.transactions(&world.accounts[0], None, None, None, None, Some(strings[1].parse().unwrap())).unwrap();
        })
        .then("I get pending transactions", |world: &mut World, _step| {
            let algod_client = world.algod_client.as_ref().unwrap();
            let _ = algod_client.pending_transactions(10).unwrap();
        })
        .when("I get the suggested params", |world: &mut World, _step| {
            let algod_client = world.algod_client.as_ref().unwrap();
            world.params_fee = Some(algod_client.transaction_params().unwrap().fee)
        })
        .when("I get the suggested fee", |world: &mut World, _step| {
            let algod_client = world.algod_client.as_ref().unwrap();
            world.fee = Some(algod_client.suggested_fee().unwrap().fee)
        })
        .then("the fee in the suggested params should equal the suggested fee", |world: &mut World, _step| {
            assert_eq!(world.params_fee.unwrap(), world.fee.unwrap())
        })
        .when("I get the status", |world: &mut World, _step| {
            let algod_client = world.algod_client.as_ref().unwrap();
            world.status = Some(algod_client.status().unwrap());
        })
        .when("I get status after this block", |world: &mut World, _step| {
            std::thread::sleep(Duration::from_secs(4));
            let algod_client = world.algod_client.as_ref().unwrap();
            let status = world.status.as_ref().unwrap();
            world.status_after = Some(algod_client.status_after_block(status.last_round).unwrap());
        })
        .then("I can get the block info", |world, _step| {
            let algod_client = world.algod_client.as_ref().unwrap();
            let status = world.status.as_ref().unwrap();
            let _ = algod_client.block(Round(status.last_round.0 + 1)).unwrap();
        })
        .given_regex(r#"default transaction with parameters (\d+) "([^"]*)""#, |world: &mut World, strings, _step| {
            let amount = MicroAlgos(strings[1].parse().unwrap());
            world.note = if strings[2] == "none" {
                Vec::new()
            } else {
                BASE64.decode(strings[2].as_bytes()).unwrap()
            };
            let algod_client = world.algod_client.as_ref().unwrap();
            let params = algod_client.transaction_params().unwrap();
            world.transaction = Some(Transaction::new_payment(
                Address::from_string(&world.accounts[0]).unwrap(),
                params.fee,
                params.last_round,
                Round(params.last_round.0 + 1000),
                world.note.clone(),
                &params.genesis_id,
                params.genesis_hash,
                Address::from_string(&world.accounts[1]).unwrap(),
                amount,
                None,
            ).unwrap());
            world.last_round = Some(params.last_round);
            world.public_key = Some(Address::from_string(&world.accounts[0]).unwrap());
        })
        .given_regex(r#"default multisig transaction with parameters (\d+) "([^"]*)""#, |world: &mut World, strings, _step| {
            let amount = MicroAlgos(strings[1].parse().unwrap());
            world.note = if strings[2] == "none" {
                Vec::new()
            } else {
                BASE64.decode(strings[2].as_bytes()).unwrap()
            };
            let algod_client = world.algod_client.as_ref().unwrap();
            let params = algod_client.transaction_params().unwrap();
            let addresses: Vec<Address> = world.accounts.iter().map(|account| Address::from_string(account).unwrap()).collect();
            let multisig = MultisigAddress::new(1, 1, &addresses).unwrap();
            world.transaction = Some(Transaction::new_payment(
                multisig.address(),
                params.fee,
                params.last_round,
                Round(params.last_round.0 + 1000),
                world.note.clone(),
                &params.genesis_id,
                params.genesis_hash,
                Address::from_string(&world.accounts[1]).unwrap(),
                amount,
                None,
            ).unwrap());
            world.last_round = Some(params.last_round);
            world.public_key = Some(Address::from_string(&world.accounts[0]).unwrap());
            world.multisig = Some(multisig);
        })
        .when("I sign the multisig transaction with kmd", |world: &mut World, _step| {
            let wallet_handle = world.wallet_handle.as_ref().expect("No wallet handle");
            let wallet_password = world.wallet_password.as_ref().expect("No wallet password");
            let transaction = world.transaction.as_ref().expect("No transaction");
            let multisig = world.multisig.as_ref().expect("No multisig");
            let public_key = world.public_key.expect("No public key");
            let kmd_client = world.kmd_client.as_ref().expect("No kmd client");
            let _ = kmd_client.import_multisig(wallet_handle, multisig.version, multisig.threshold, &multisig.public_keys).unwrap();
            world.signed_transaction_bytes = kmd_client.sign_multisig_transaction(wallet_handle, wallet_password, transaction, Ed25519PublicKey(public_key.bytes), None).unwrap().multisig;
        })
        .when("I get versions with kmd", |world: &mut World, _step| {
            let kmd_client = world.kmd_client.as_ref().unwrap();
            world.versions = kmd_client.versions().unwrap().versions;
        })
        .when("I get versions with algod", |world: &mut World, _step| {
            let algod_client = world.algod_client.as_ref().unwrap();
            world.versions = algod_client.versions().unwrap().versions;
        })
        .then("v1 should be in the versions", |world: &mut World, _step| {
            assert!(world.versions.contains(&"v1".to_string()))
        })
        .when("I create a wallet", |world: &mut World, _step| {
            let wallet_name = "Walletrust";
            let wallet_password = "";
            let response = world.kmd_client.as_ref().unwrap().create_wallet(wallet_name, wallet_password, "sqlite", MasterDerivationKey([0; 32])).unwrap();
            world.wallet_name = Some(wallet_name.to_string());
            world.wallet_password = Some(wallet_password.to_string());
            world.wallet_id = Some(response.wallet.id);
        })
        .then("the wallet should exist", |world: &mut World, _step| {
            let mut exists = false;
            let wallet_name = world.wallet_name.as_ref().unwrap();
            let kmd_client = world.kmd_client.as_ref().unwrap();
            for wallet in kmd_client.list_wallets().unwrap().wallets {
                if &wallet.name == wallet_name {
                    exists = true;
                }
            }
            assert!(exists);
        })
        .when("I get the wallet handle", |world: &mut World, _step| {
            let wallet_id = world.wallet_id.as_ref().unwrap();
            let wallet_password = world.wallet_password.as_ref().unwrap();
            let kmd_client = world.kmd_client.as_ref().unwrap();
            world.wallet_handle = Some(kmd_client.init_wallet_handle(wallet_id, wallet_password).unwrap().wallet_handle_token);
        })
        .then("I can get the master derivation key", |world: &mut World, _step| {
            let wallet_handle = world.wallet_handle.as_ref().unwrap();
            let wallet_password = world.wallet_password.as_ref().unwrap();
            let kmd_client = world.kmd_client.as_ref().unwrap();
            let _ = kmd_client.export_master_derivation_key(wallet_handle, wallet_password).unwrap();
        })
        .when("I rename the wallet", |world: &mut World, _step| {
            let wallet_id = world.wallet_id.as_ref().unwrap();
            let wallet_password = world.wallet_password.as_ref().unwrap();
            let new_name = "Walletrust_new";
            let kmd_client = world.kmd_client.as_ref().unwrap();
            let _ = kmd_client.rename_wallet(wallet_id, wallet_password, new_name).unwrap();
            world.wallet_name = Some(new_name.to_string());
        })
        .then("I can still get the wallet information with the same handle", |world: &mut World, _step| {
            let wallet_handle = world.wallet_handle.as_ref().unwrap();
            let kmd_client = world.kmd_client.as_ref().unwrap();
            let name = kmd_client.get_wallet(wallet_handle).unwrap().wallet_handle.wallet.name;
            assert_eq!(Some(name), world.wallet_name);
        })
        .when("I renew the wallet handle", |world: &mut World, _step| {
            let wallet_handle = world.wallet_handle.as_ref().unwrap();
            let kmd_client = world.kmd_client.as_ref().unwrap();
            let _ = kmd_client.renew_wallet_handle(wallet_handle).unwrap();
        })
        .when("I release the wallet handle", |world: &mut World, _step| {
            let wallet_handle = world.wallet_handle.as_ref().unwrap();
            let kmd_client = world.kmd_client.as_ref().unwrap();
            let _ = kmd_client.release_wallet_handle(wallet_handle).unwrap();
        })
        .then("the wallet handle should not work", |world: &mut World, _step| {
            let wallet_handle = world.wallet_handle.as_ref().unwrap();
            let kmd_client = world.kmd_client.as_ref().unwrap();
            assert!(kmd_client.renew_wallet_handle(wallet_handle).is_err());
        })
        .when("I generate a key using kmd", |world: &mut World, _step| {
            let wallet_handle = world.wallet_handle.as_ref().unwrap();
            let kmd_client = world.kmd_client.as_ref().unwrap();
            let public_key = kmd_client.generate_key(wallet_handle).unwrap().address;
            world.public_key = Some(Address::from_string(&public_key).unwrap())
        })
        .then("the key should be in the wallet", |world: &mut World, _step| {
            let wallet_handle = world.wallet_handle.as_ref().unwrap();
            let kmd_client = world.kmd_client.as_ref().unwrap();
            assert!(kmd_client.list_keys(wallet_handle).unwrap().addresses.contains(&world.public_key.unwrap().encode_string()));
        })
        .when("I delete the key", |world: &mut World, _step| {
            let wallet_handle = world.wallet_handle.as_ref().unwrap();
            let wallet_password = world.wallet_password.as_ref().unwrap();
            let public_key = world.public_key.unwrap();
            let kmd_client = world.kmd_client.as_ref().unwrap();
            let _ = kmd_client.delete_key(wallet_handle, wallet_password, &public_key.encode_string()).unwrap();
        })
        .then("the key should not be in the wallet", |world: &mut World, _step| {
            let wallet_handle = world.wallet_handle.as_ref().unwrap();
            let kmd_client = world.kmd_client.as_ref().unwrap();
            assert!(!kmd_client.list_keys(wallet_handle).unwrap().addresses.contains(&world.public_key.unwrap().encode_string()));
        })
        .then("I get account information", |world: &mut World, _step| {
            let algod_client = world.algod_client.as_ref().unwrap();
            let _ = algod_client.account_information(&world.accounts[0]).unwrap();
        })
        .then("I can get account information", |world: &mut World, _step| {
            let wallet_handle = world.wallet_handle.as_ref().unwrap();
            let wallet_password = world.wallet_password.as_ref().unwrap();
            let address = world.public_key.unwrap().encode_string();
            let kmd_client = world.kmd_client.as_ref().unwrap();
            let algod_client = world.algod_client.as_ref().unwrap();
            let _ = algod_client.account_information(&address).unwrap();
            let _ = kmd_client.delete_key(wallet_handle, wallet_password, &address).unwrap();
        })
        .when("I import the key", |world: &mut World, _step| {
            let kmd_client = world.kmd_client.as_ref().unwrap();
            let wallet_handle = world.wallet_handle.as_ref().unwrap();
            let private_key = world.account.as_ref().unwrap().seed;
            let _ = kmd_client.import_key(wallet_handle, private_key).unwrap();
        })
        .then("the private key should be equal to the exported private key", |world: &mut World, _step| {
            let wallet_handle = world.wallet_handle.as_ref().unwrap();
            let wallet_password = world.wallet_password.as_ref().unwrap();
            let address = world.public_key.unwrap().encode_string();
            let private_key = world.account.as_ref().unwrap().seed;
            let kmd_client = world.kmd_client.as_ref().unwrap();
            let exported = kmd_client.export_key(wallet_handle, wallet_password, &address).unwrap().private_key;
            assert_eq!(&exported[..32], &private_key);
            let _ = kmd_client.delete_key(wallet_handle, wallet_password, &address).unwrap();
        })
        .when("I get the private key", |world: &mut World, _step| {
            let wallet_handle = world.wallet_handle.as_ref().expect("No wallet handle");
            let wallet_password = world.wallet_password.as_ref().expect("No wallet password");
            let address = world.public_key.expect("No public key").encode_string();
            let kmd_client = world.kmd_client.as_ref().expect("No kmd client");
            let exported_seed = kmd_client.export_key(wallet_handle, wallet_password, &address).unwrap().private_key;
            let mut seed = [0; 32];
            seed.copy_from_slice(&exported_seed[..32]);
            world.account = Some(Account::from_seed(seed));
        })
        .when("I sign the transaction with kmd", |world: &mut World, _step| {
            let wallet_handle = world.wallet_handle.as_ref().unwrap();
            let wallet_password = world.wallet_password.as_ref().unwrap();
            let transaction = world.transaction.as_ref().unwrap();
            let kmd_client = world.kmd_client.as_ref().unwrap();
            world.signed_transaction_bytes = kmd_client.sign_transaction(wallet_handle, wallet_password, transaction).unwrap().signed_transaction;
        })
        .then("the signed transaction should equal the kmd signed transaction", |world: &mut World, _step| {
            let local_bytes = rmp_serde::to_vec_named(world.signed_transaction.as_ref().unwrap()).unwrap();
            assert_eq!(BASE64.encode(&world.signed_transaction_bytes), BASE64.encode(&local_bytes));
        })
        .then("the multisig transaction should equal the kmd signed multisig transaction", |world: &mut World, _step| {
            let local_bytes = rmp_serde::to_vec_named(world.signed_transaction.as_ref().unwrap().multisig.as_ref().unwrap()).unwrap();
            assert_eq!(BASE64.encode(&world.signed_transaction_bytes), BASE64.encode(&local_bytes));

            let multisig = world.multisig.as_ref().unwrap();
            let wallet_handle = world.wallet_handle.as_ref().unwrap();
            let wallet_password = world.wallet_password.as_ref().unwrap();
            let kmd_client = world.kmd_client.as_ref().unwrap();
            let _ = kmd_client.delete_multisig(wallet_handle, wallet_password, &multisig.address().encode_string()).unwrap();
        })
        .when("I send the transaction", |world: &mut World, _step| {
            let algod_client = world.algod_client.as_ref().unwrap();
            let signed_transaction = world.signed_transaction.as_ref().unwrap();
            let bytes = rmp_serde::to_vec_named(signed_transaction).unwrap();
            world.transaction_id = Some(algod_client.raw_transaction(&bytes).unwrap().tx_id);
        })
        .when("I send the multisig transaction", |world: &mut World, _step| {
            let algod_client = world.algod_client.as_ref().unwrap();
            let signed_transaction = world.signed_transaction.as_ref().unwrap();
            let bytes = rmp_serde::to_vec_named(signed_transaction).unwrap();
            world.err = algod_client.raw_transaction(&bytes).is_err();
        })
        .then("the transaction should go through", |world: &mut World, _step| {
            let algod_client = world.algod_client.as_ref().unwrap();
            let public_key = world.public_key.unwrap().encode_string();
            let transaction_id = world.transaction_id.as_ref().unwrap();
            assert_eq!(algod_client.pending_transaction_information(transaction_id).unwrap().from, public_key);
            let _ = algod_client.status_after_block(Round(world.last_round.unwrap().0 + 2));
            assert_eq!(algod_client.transaction_information(&public_key, transaction_id).unwrap().from, public_key);
            assert_eq!(algod_client.transaction(transaction_id).unwrap().from, public_key);
        })
        .then("the transaction should not go through", |world: &mut World, _step| {
            assert!(world.err)
        })
        .then("I can get the transaction by ID", |world: &mut World, _step| {
            let algod_client = world.algod_client.as_ref().unwrap();
            let transaction_id = world.transaction_id.as_ref().unwrap();
            let public_key = world.public_key.unwrap().encode_string();
            let _ = algod_client.status_after_block(Round(world.last_round.unwrap().0 + 2)).unwrap();
            assert_eq!(algod_client.transaction(transaction_id).unwrap().from, public_key);
        })
        .when("I import the multisig", |world: &mut World, _step| {
            let multisig = world.multisig.as_ref().unwrap();
            let wallet_handle = world.wallet_handle.as_ref().unwrap();
            let kmd_client = world.kmd_client.as_ref().unwrap();
            let _ = kmd_client.import_multisig(wallet_handle, multisig.version, multisig.threshold, &multisig.public_keys).unwrap();
        })
        .when("I export the multisig", |world: &mut World, _step| {
            let multisig = world.multisig.as_ref().unwrap();
            let wallet_handle = world.wallet_handle.as_ref().unwrap();
            let kmd_client = world.kmd_client.as_ref().unwrap();
            world.public_keys = kmd_client.export_multisig(wallet_handle, &multisig.address().encode_string()).unwrap().pks;
        })
        .when("I delete the multisig", |world: &mut World, _step| {
            let multisig = world.multisig.as_ref().unwrap();
            let wallet_handle = world.wallet_handle.as_ref().unwrap();
            let wallet_password = world.wallet_password.as_ref().unwrap();
            let kmd_client = world.kmd_client.as_ref().unwrap();
            let _ = kmd_client.delete_multisig(wallet_handle, wallet_password, &multisig.address().encode_string()).unwrap();
        })
        .then("the multisig should equal the exported multisig", |world: &mut World, _step| {
            let multisig = world.multisig.as_ref().unwrap();
            for (left, right) in multisig.public_keys.iter().zip(&world.public_keys) {
                assert_eq!(left, right);
            }
        })
        .then("the multisig should be in the wallet", |world: &mut World, _step| {
            let wallet_handle = world.wallet_handle.as_ref().unwrap();
            let kmd_client = world.kmd_client.as_ref().unwrap();
            let addresses = kmd_client.list_multisig(wallet_handle).unwrap().addresses;
            assert!(addresses.contains(&world.multisig.as_ref().unwrap().address().encode_string()));
        })
        .then("the multisig should not be in the wallet", |world: &mut World, _step| {
            let wallet_handle = world.wallet_handle.as_ref().unwrap();
            let kmd_client = world.kmd_client.as_ref().unwrap();
            let addresses = kmd_client.list_multisig(wallet_handle).unwrap().addresses;
            assert!(!addresses.contains(&world.multisig.as_ref().unwrap().address().encode_string()));
        })
        .when("I generate a key", |world: &mut World, _step| {
            let account = Account::generate();
            world.public_key = Some(account.address());
            world.address = Some(account.address().encode_string());
            world.account = Some(account);
        })
        .when("I decode the address", |world: &mut World, _step| {
            world.public_key = Some(Address::from_string(&world.address.as_ref().unwrap()).unwrap());
            world.old_address = world.address.clone();
        })
        .when("I encode the address", |world: &mut World, _step| {
            world.address = Some(world.public_key.unwrap().encode_string());
        })
        .then("the address should still be the same", |world: &mut World, _step| {
            assert_eq!(world.old_address, world.address);
        })
        .given_regex(r#"mnemonic for private key "([^"]*)"#, |world: &mut World, strings, _step| {
            let account = Account::from_seed(mnemonic::to_key(&strings[1]).unwrap());
            world.public_key = Some(account.address());
            world.account = Some(account)
        })
        .when("I convert the private key back to a mnemonic", |world: &mut World, _step| {
            world.new_mnemonic = Some(mnemonic::from_key(&world.account.as_ref().unwrap().seed).unwrap());
        })
        .then_regex(r#"^the mnemonic should still be the same as "([^"]*)""#, |world: &mut World, strings, _step| {
            assert_eq!(world.new_mnemonic.as_ref().unwrap(), &strings[1])
        })
        .given_regex(r#"^mnemonic for master derivation key "([^"]*)""#, |world: &mut World, strings, _step| {
            world.mdk = Some(mnemonic::to_key(&strings[1]).unwrap());
        })
        .when("I convert the master derivation key back to a mnemonic", |world: &mut World, _step| {
            world.new_mnemonic = Some(mnemonic::from_key(&world.mdk.unwrap()).unwrap());
        })
        .given_regex(r#"payment transaction parameters (\d+) (\d+) (\d+) "([^"]*)" "([^"]*)" "([^"]*)" (\d+) "([^"]*)" "([^"]*)""#, |world: &mut World, strings, _step| {
            world.fee = Some(MicroAlgos(strings[1].parse().unwrap()));
            world.first_valid = Some(Round(strings[2].parse().unwrap()));
            world.last_valid = Some(Round(strings[3].parse().unwrap()));
            let mut genesis_hash = [0; 32];
            genesis_hash.copy_from_slice(&BASE64.decode(strings[4].as_bytes()).unwrap());
            world.genesis_hash = Some(HashDigest(genesis_hash));
            world.receiver = Some(Address::from_string(&strings[5]).unwrap());
            if strings[6] != "none" {
                world.close = Some(Address::from_string(&strings[6]).unwrap());
            }
            world.amount = Some(MicroAlgos(strings[7].parse().unwrap()));
            if strings[8] != "none" {
                world.genesis_id = strings[8].clone();
            }
            if strings[9] != "none" {
                world.note = BASE64.decode(strings[9].as_bytes()).unwrap();
            }
        })
        .given_regex(r#"key registration transaction parameters (\d+) (\d+) (\d+) "([^"]*)" "([^"]*)" "([^"]*)" (\d+) (\d+) (\d+) "([^"]*)" "([^"]*)"#, |world: &mut World, strings, _step| {
            world.fee = Some(MicroAlgos(strings[1].parse().unwrap()));
            world.first_valid = Some(Round(strings[2].parse().unwrap()));
            world.last_valid = Some(Round(strings[3].parse().unwrap()));
            let mut genesis_hash = [0; 32];
            genesis_hash.copy_from_slice(&BASE64.decode(strings[4].as_bytes()).unwrap());
            world.genesis_hash = Some(HashDigest(genesis_hash));

            let mut vote_pk = [0; 32];
            vote_pk.copy_from_slice(&BASE64.decode(strings[5].as_bytes()).unwrap());
            world.vote_pk = Some(VotePK(vote_pk));
            let mut vrf_pk = [0; 32];
            vrf_pk.copy_from_slice(&BASE64.decode(strings[6].as_bytes()).unwrap());
            world.vrf_pk = Some(VRFPK(vrf_pk));
            world.vote_first = Some(Round(strings[7].parse().unwrap()));
            world.vote_last = Some(Round(strings[8].parse().unwrap()));
            world.vote_key_dilution = Some(strings[9].parse().unwrap());

            if strings[10] != "none" {
                world.genesis_id = strings[10].clone();
            }
            if strings[11] != "none" {
                world.note = BASE64.decode(strings[11].as_bytes()).unwrap();
            }
        })
        .when("I create the payment transaction", |world: &mut World, _step| {
            world.transaction = Some(Transaction::new_payment(
                world.public_key.expect("No public key"),
                world.fee.expect("No fee"),
                world.first_valid.expect("No first valid"),
                world.last_valid.expect("No last valid"),
                world.note.clone(),
                &world.genesis_id,
                world.genesis_hash.expect("No genesis hash"),
                world.receiver.expect("No receiver"),
                world.amount.expect("No amount"),
                world.close,
            ).unwrap());
        })
        .when("I create the flat fee payment transaction", |world: &mut World, _step| {
            world.transaction = Some(Transaction::new_payment_flat_fee(
                world.public_key.expect("No public key"),
                world.fee.expect("No fee"),
                world.first_valid.expect("No first valid"),
                world.last_valid.expect("No last valid"),
                world.note.clone(),
                &world.genesis_id,
                world.genesis_hash.expect("No genesis hash"),
                world.receiver.expect("No receiver"),
                world.amount.expect("No amount"),
                world.close,
            ).unwrap());
        })
        .when("I create the multisig payment transaction", |world: &mut World, _step| {
            world.transaction = Some(Transaction::new_payment(
                world.multisig.as_ref().expect("No multisig address").address(),
                world.fee.expect("No fee"),
                world.first_valid.expect("No first valid"),
                world.last_valid.expect("No last valid"),
                world.note.clone(),
                &world.genesis_id,
                world.genesis_hash.expect("No genesis hash"),
                world.receiver.expect("No receiver"),
                world.amount.expect("No amount"),
                world.close,
            ).unwrap());
        })
        .when("I create the key registration transaction", |world: &mut World, _step| {
            world.transaction = Some(Transaction::new_key_registration(
                world.public_key.expect("No public key"),
                world.fee.expect("No fee"),
                world.first_valid.expect("No first valid"),
                world.last_valid.expect("No last valid"),
                world.note.clone(),
                &world.genesis_id,
                world.genesis_hash.expect("No genesis hash"),
                world.vote_pk.expect("No vote public key"),
                world.vrf_pk.expect("No VRFPK"),
                world.vote_first.expect("No vote first"),
                world.vote_last.expect("No vote last"),
                world.vote_key_dilution.expect("No vote key dilution"),
            ).unwrap());
        })
        .when("I sign the transaction with the private key", |world: &mut World, _step| {
            world.signed_transaction = Some(world.account.as_ref().unwrap().sign_transaction(world.transaction.as_ref().unwrap()).expect("Failed to sign transaction"))
        })
        .when("I sign the multisig transaction with the private key", |world: &mut World, _step| {
            world.signed_transaction = Some(world.account.as_ref().unwrap().sign_multisig_transaction(world.multisig.clone().unwrap(), world.transaction.as_ref().unwrap()).expect("Failed to sign transaction"))
        })
        .then_regex(r#"the signed transaction should equal the golden "([^"]*)""#, |world: &mut World, strings, _step| {
            let bytes = rmp_serde::to_vec_named(world.signed_transaction.as_ref().unwrap()).unwrap();
            assert_eq!(BASE64.encode(&bytes), strings[1])
        })
        .then_regex(r#"the multisig transaction should equal the golden "([^"]*)""#, |world: &mut World, strings, _step| {
            let bytes = rmp_serde::to_vec_named(world.signed_transaction.as_ref().unwrap()).unwrap();
            assert_eq!(BASE64.encode(&bytes), strings[1])
        })
        .given_regex(r#"multisig addresses "([^"]*)""#, |world: &mut World, strings, _step| {
            let addresses: Vec<Address> = strings[1].split(' ').map(|s| Address::from_string(s).unwrap()).collect();
            world.multisig = Some(MultisigAddress::new(1, 2, &addresses).unwrap());
        })
        .then_regex(r#"the multisig address should equal the golden "([^"]*)""#, |world: &mut World, strings, _step| {
            let msig_address = world.multisig.as_ref().unwrap().address();
            assert_eq!(msig_address.encode_string(), strings[1])
        })
        .given_regex(r#"encoded multisig transaction "([^"]*)""#, |world: &mut World, strings, _step| {
            let bytes = BASE64.decode(strings[1].as_bytes()).expect("Failed to decode from base64");
            let signed_transaction: SignedTransaction = rmp_serde::from_read_ref(&bytes).expect("Failed to decode");
            let (version, threshold, addresses) = {
                let multisig = signed_transaction.multisig.as_ref().unwrap();
                let addresses: Vec<Address> = multisig.subsigs.iter().map(|subsig| Address::new(subsig.key.0)).collect();
                (multisig.version, multisig.threshold, addresses)
            };
            world.multisig = Some(MultisigAddress::new(version, threshold, &addresses).unwrap());
            world.signed_transaction = Some(signed_transaction);
        })
        .when("I append a signature to the multisig transaction", |world: &mut World, _step| {
            let multisig = world.multisig.clone().unwrap();
            let signed_transaction = world.signed_transaction.as_ref().unwrap();
            world.signed_transaction = Some(world.account.as_ref().unwrap().append_multisig_transaction(multisig, signed_transaction).unwrap())
        })
        .given_regex(r#"encoded multisig transactions "([^"]*)""#, |world: &mut World, strings, _step| {
            let transactions: Vec<SignedTransaction> = strings[1].split(' ').map(|encoded| {
                let bytes = BASE64.decode(encoded.as_bytes()).unwrap();
                rmp_serde::from_read_ref(&bytes).unwrap()
            }).collect();
            world.signed_transactions = Some(transactions);
        })
        .when("I merge the multisig transactions", |world: &mut World, _step| {
            let signed_transaction = Account::merge_multisig_transactions(&world.signed_transactions.as_ref().unwrap()).unwrap();
            world.signed_transaction = Some(signed_transaction);
        })
        .when_regex(r"^I convert (\d+) microalgos to algos and back", |world: &mut World, strings, _step| {
            let micro_algos: MicroAlgos = MicroAlgos(strings[1].parse().unwrap());
            world.micro_algos = Some(MicroAlgos::from_algos(micro_algos.to_algos()));
        })
        .then_regex(r"^it should still be the same amount of microalgos (\d+)", |world: &mut World, strings, _step| {
            let micro_algos: MicroAlgos = MicroAlgos(strings[1].parse().unwrap());
            assert_eq!(world.micro_algos.unwrap(), micro_algos);
        })
        .when("I create a bid", |world: &mut World, _step| {
            let account = Account::generate();
            world.public_key = Some(account.address());
            world.address = Some(account.address().encode_string());
            world.bid = Some(Bid {
                auction_key: account.address(),
                bidder_key: account.address(),
                auction_id: 1,
                bid_currency: 2,
                bid_id: 3,
                max_price: 4,
            });
            world.account = Some(account);
        })
        .when("I sign the bid", |world: &mut World, _step| {
            let account = world.account.as_ref().unwrap();
            world.signed_bid = Some(account.sign_bid(world.bid.unwrap()));
            world.old_bid = world.signed_bid;
        })
        .when("I encode and decode the bid", |world: &mut World, _step| {
            let encoded = rmp_serde::to_vec_named(&world.signed_bid.unwrap()).unwrap();
            let signed_bid = rmp_serde::from_read_ref(&encoded).unwrap();
            world.signed_bid = Some(signed_bid);
        })
        .then("the bid should still be the same", |world: &mut World, _step| {
            assert_eq!(world.signed_bid.unwrap(), world.old_bid.unwrap());
        })
        .when_regex(r#"I read a transaction "([^"]*)" from file "([^"]*)""#, |world: &mut World, strings, _step| {
            world.num = strings[2].clone();
            let path = std::env::current_dir().expect("Couldn't get current dir").parent().unwrap().join(format!("temp/raw{}.tx", strings[2]));
            let f = File::open(path).unwrap();
            world.signed_transaction = Some(rmp_serde::from_read(f).unwrap());
        })
        .when("I write the transaction to file", |world: &mut World, _step| {
            let path = std::env::current_dir().expect("Couldn't get current dir").parent().unwrap().join(format!("temp/raw{}.tx", world.num));
            let data = rmp_serde::to_vec_named(world.signed_transaction.as_ref().unwrap()).unwrap();
            let mut f = File::create(path).unwrap();
            let _ = f.write_all(&data).unwrap();
        })
        .then("the transaction should still be the same", |world: &mut World, _step| {
            let path = std::env::current_dir().expect("Couldn't get current dir");
            let path = path.parent().unwrap();
            let f = File::open(path.join(format!("temp/raw{}.tx", world.num))).unwrap();
            let new: SignedTransaction = rmp_serde::from_read(f).unwrap();

            let f = File::open(path.join(format!("temp/old{}.tx", world.num))).unwrap();
            let old: SignedTransaction = rmp_serde::from_read(f).unwrap();

            assert_eq!(new, old);
        })
        .then("I do my part", |world: &mut World, _step| {
            let path = std::env::current_dir().expect("Couldn't get current dir").parent().unwrap().join(format!("temp/txn.tx"));
            let mut f = File::open(&path).unwrap();
            let mut bytes = Vec::new();
            let _ = f.read_to_end(&mut bytes).unwrap();
            let signed_transaction: SignedTransaction = rmp_serde::from_read_ref(&bytes).unwrap();
            let transaction = signed_transaction.transaction;

            let wallet_handle = world.wallet_handle.as_ref().expect("No wallet handle");
            let wallet_password = world.wallet_password.as_ref().expect("No wallet password");
            let address = transaction.sender.encode_string();
            let kmd_client = world.kmd_client.as_ref().expect("No kmd client");
            let exported_seed = kmd_client.export_key(wallet_handle, wallet_password, &address).unwrap().private_key;
            let mut seed = [0; 32];
            seed.copy_from_slice(&exported_seed[..32]);
            let account = Account::from_seed(seed);
            let signed_transaction = account.sign_transaction(&transaction).unwrap();

            let data = rmp_serde::to_vec_named(&signed_transaction).unwrap();
            let mut f = File::create(&path).unwrap();
            let _ = f.write_all(&data).unwrap();

            world.account = Some(account);
        })
    ;
    builder.build()
}

/*
cucumber! {
    features: "./features",
    world: World,
    steps: &[crate::steps]
}
*/
#[allow(unused_imports)]
fn main() {
    use std::path::Path;
    use cucumber::{CucumberBuilder, Scenario, Steps, DefaultOutput, OutputVisitor};

    let output = DefaultOutput::new();
    let instance = {
        let mut instance = CucumberBuilder::new(output);

        instance
            .features(<[_]>::into_vec(Box::new([(Path::new("./features").to_path_buf())])))
            .steps(Steps::combine((&[crate::steps]
            ).iter().map(|f| f())));
        instance
    };

    let options = cucumber::cli::make_app().unwrap();
    println!("{:?}", options.tag);

    let res = instance.command_line();

    if !res {
        std::process::exit(1);
    }
}
