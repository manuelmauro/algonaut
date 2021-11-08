// simple smart contract based on https://developer.algorand.org/tutorials/writing-simple-smart-contract/ 
use algonaut_client::algod::v2::Client as AlgodClient;
use algonaut_client::indexer::v2::message::{Account, QueryAccount};
use algonaut_client::indexer::v2::Client as IndexerClient;
use algonaut_client::{Algod, Indexer, Kmd};
use algonaut_core::{Address, LogicSignature, MicroAlgos};
use algonaut_transaction::{Pay, SignedTransaction, Txn};
use data_encoding::BASE64;
use std::env;
use std::process::exit;

const TEAL_PROGRAM: &str = "
// Check if fee is reasonable
// In this case 10,000 microalgos
txn Fee
int 10000
<=

// Check the length of the passphrase is correct
arg 0
len
int 73
==
&&

// The SHA256 value of the passphrase
arg 0
sha256
byte base64 30AT2gOReDBdJmLBO/DgvjC6hIXgACecTpFDcP1bJHU=
==
&&

// Make sure the CloseRemainderTo is not set
txn CloseRemainderTo
txn Receiver
==
&&";

#[allow(dead_code)]
#[derive(Default)]
struct EnvironmentConfig {
    algod_address: String,
    algod_token: String,
    kmd_address: String,
    kmd_token: String,
    indexer_address: String,
}

pub enum AlgodEnvironment {
    Sandbox,
    PrivNet,
    TestNet,
    MainNet,
}

fn get_val(key: String) -> String {
    match env::var(&key) {
        Ok(val) => return val,
        Err(e) => {
            println!("error getting env var: {} {}", key, e);
            exit(1)
        }
    }
}

impl AlgodEnvironment {
    fn get_config(&self) -> EnvironmentConfig {
        let network: String = match self {
            AlgodEnvironment::Sandbox => "SANDBOX".into(),
            AlgodEnvironment::PrivNet => "PRIVNET".into(),
            AlgodEnvironment::TestNet => "TESTNET".into(),
            AlgodEnvironment::MainNet => "MAINNET".into(),
        };

        EnvironmentConfig {
            algod_address: get_val(format!("{}_ALGOD_ADDRESS", network)),
            algod_token: get_val(format!("{}_ALGOD_TOKEN", network)),
            kmd_address: get_val(format!("{}_KMD_ADDRESS", network)),
            kmd_token: get_val(format!("{}_KMD_TOKEN", network)),
            indexer_address: get_val(format!("{}_INDEXER_ADDRESS", network)),
        }
    }
}

fn get_balance(client: &AlgodClient, address: &str) -> u64 {
    client
        .account_information(address)
        .unwrap()
        .amount_without_pending_rewards
}

fn main() {
    let algod_config: EnvironmentConfig = AlgodEnvironment::Sandbox.get_config();
    let passphrase = get_val("PASSPHRASE".to_string());

    // build clients
    let algod_client: AlgodClient = Algod::new()
        .bind(&algod_config.algod_address)
        .auth(&algod_config.algod_token)
        .client_v2()
        .unwrap();

    let indexer_client: IndexerClient = Indexer::new()
        .bind(&algod_config.indexer_address)
        .client_v2()
        .unwrap();

    let kmd = Kmd::new()
        .bind(&algod_config.kmd_address)
        .auth(&algod_config.kmd_token)
        .client_v1()
        .unwrap();

    // compile teal program
    let contract = algod_client.compile_teal(TEAL_PROGRAM.into()).unwrap();

    // obtain a handle to our wallet
    let list_response = kmd.list_wallets().unwrap();
    let wallet_id = match list_response
        .wallets
        .into_iter()
        .find(|wallet| wallet.name == "unencrypted-default-wallet")
    {
        Some(wallet) => wallet.id,
        None => String::new(),
    };
    let init_response = kmd.init_wallet_handle(&wallet_id, "").unwrap();
    let wallet_handle_token = init_response.wallet_handle_token;

    // for this example, arbitrarily choose the first 2 accounts returned using deafult network
    // config. make sure this way of determining accounts makes sense for the environment.
    let query: QueryAccount = QueryAccount {
        application_id: None,
        asset_id: None,
        auth_addr: None,
        currency_greater_than: None,
        currency_less_than: None,
        limit: None,
        next: None,
        round: None,
    };
    let accounts: Vec<Account> = indexer_client.accounts(&query).unwrap().accounts;
    let alice: &Account = &accounts[0];
    let bob: &Account = &accounts[1];

    println!("addresses");
    println!("alice {}", alice.address);
    println!("bob {}", bob.address);
    println!("contract {}\n", contract.hash);

    println!("starting balances");
    println!("{} alice", get_balance(&algod_client, &alice.address));
    println!("{} bob", get_balance(&algod_client, &bob.address));
    println!("{} contract\n", get_balance(&algod_client, &contract.hash));

    let params = algod_client.transaction_params().unwrap();

    let t = Txn::new()
        .sender(alice.address.parse().unwrap())
        .first_valid(params.last_round)
        .last_valid(params.last_round + 1000)
        .genesis_id(params.genesis_id)
        .genesis_hash(params.genesis_hash)
        .fee(MicroAlgos(10_000))
        .payment(
            Pay::new()
                .amount(MicroAlgos(1_000_000))
                .to(contract.hash.parse().unwrap())
                .build(),
        )
        .build();

    // we need to sign the transaction to prove that we own the sender address
    let sign_response = kmd.sign_transaction(&wallet_handle_token, "", &t).unwrap();

    // broadcast the transaction to the network
    let send_response = algod_client
        .broadcast_raw_transaction(&sign_response.signed_transaction)
        .unwrap();

    println!("alice->contract transaction id: {}\n", send_response.tx_id);

    // wait for transaction to finalize
    loop {
        let txn_state = algod_client
            .pending_transaction_with_id(&send_response.tx_id)
            .unwrap();

        if let Some(_) = txn_state.confirmed_round {
            break;
        }

        println!(
            "txn {}... not confirmed; sleep 2s...",
            &send_response.tx_id[..5]
        );
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    println!("\nbalances after contract funded");
    println!("{} alice", get_balance(&algod_client, &alice.address));
    println!("{} bob", get_balance(&algod_client, &bob.address));
    println!("{} contract\n", get_balance(&algod_client, &contract.hash));

    // Next step is to provide an argument (password) to the contract account so that it will
    // release its funds to the `close-to` address:
    // ${gcmd} clerk send \
    // --amount 30000 \
    // --from-program ./passphrase.teal \
    // --close-to "${bob}" \
    // --to "${bob}" \
    // --argb64 "$(echo -n ${PASSPHRASE} | base64 -w 0)" \
    // --out out.txn
    println!("closing contract by providing password...");
    let passphrase_arg = passphrase.as_bytes().to_owned();
    let program_bytes = BASE64.decode(contract.result.as_bytes()).unwrap();
    let lsig = LogicSignature {
        logic: program_bytes,
        sig: None,
        msig: None,
        args: vec![passphrase_arg],
    };

    let contract_address: Address = contract.hash.parse().unwrap();
    let bob_address: Address = bob.address.parse().unwrap();

    let params = algod_client.transaction_params().unwrap();

    let t = Txn::new()
        .sender(contract_address)
        .first_valid(params.last_round)
        .last_valid(params.last_round + 10)
        .genesis_id(params.genesis_id)
        .genesis_hash(params.genesis_hash)
        .fee(MicroAlgos(10_000))
        .payment(
            Pay::new()
                .amount(MicroAlgos(30_000))
                .to(bob_address)
                .close_remainder_to(bob_address)
                .build(),
        )
        .build();

    let signed_transaction = SignedTransaction {
        sig: None,
        multisig: None,
        logicsig: Some(lsig),
        transaction: t,
        transaction_id: "".to_owned(),
    };

    let transaction_bytes = rmp_serde::to_vec_named(&signed_transaction).unwrap();
    let send_response = algod_client
        .broadcast_raw_transaction(&transaction_bytes)
        .unwrap();

    // wait for transaction to finalize
    loop {
        let txn_state = algod_client
            .pending_transaction_with_id(&send_response.tx_id)
            .unwrap();

        if let Some(_) = txn_state.confirmed_round {
            break;
        }

        println!(
            "txn {}... not confirmed; sleep 2s...",
            &send_response.tx_id[..5]
        );
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    println!("\nbalances after contract closed");
    println!("{} alice", get_balance(&algod_client, &alice.address));
    println!("{} bob", get_balance(&algod_client, &bob.address));
    println!("{} contract", get_balance(&algod_client, &contract.hash));
}
