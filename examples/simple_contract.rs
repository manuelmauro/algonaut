//!
//! Implementation of the "Writing a Simple Smart Contract" tutorial in rust using
//! manuelmauro/algonaut
//! https://developer.algorand.org/tutorials/writing-simple-smart-contract/
//!
use algonaut::algod::v2::Algod;
use algonaut::algod::AlgodBuilder;
use algonaut::indexer::IndexerBuilder;
use algonaut::kmd::KmdBuilder;
use algonaut_core::{Address, MicroAlgos};
use algonaut_model::indexer::v2::QueryAccount;
use algonaut_transaction::account::ContractAccount;
use algonaut_transaction::{Pay, TxnBuilder};
use dotenv::dotenv;
use std::env;
use std::error::Error;
use std::str::FromStr;

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

async fn get_balance(client: &Algod, address: &Address) -> MicroAlgos {
    client.account_information(address).await.unwrap().amount
}

async fn wait_for_txn(client: &Algod, txn_id: &str) {
    loop {
        let txn_state = client.pending_transaction_with_id(txn_id).await.unwrap();

        if let Some(_) = txn_state.confirmed_round {
            break;
        }

        println!("txn {} not confirmed; sleep 2s...", &txn_id[..5]);
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}

async fn print_status(client: &Algod, alice: &Address, bob: &Address, contract: &Address) {
    println!("alice {}", get_balance(client, alice).await);
    println!("bob {}", get_balance(client, bob).await);
    println!("contract {}\n", get_balance(client, contract).await);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load variables in .env
    dotenv().ok();

    // build clients
    let algod = AlgodBuilder::new()
        .bind(env::var("ALGOD_URL")?.as_ref())
        .auth(env::var("ALGOD_TOKEN")?.as_ref())
        .build_v2()?;

    let indexer = IndexerBuilder::new()
        .bind(env::var("INDEXER_URL")?.as_ref())
        .build_v2()?;

    let kmd = KmdBuilder::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .build_v1()?;

    let compiled_teal = algod
        .compile_teal(String::from(TEAL_PROGRAM).as_bytes())
        .await?;

    let contract_account = ContractAccount::new(compiled_teal);

    let accounts = indexer
        .accounts(&QueryAccount::default())
        .await
        .unwrap()
        .accounts;

    // pick any 2 accounts which show up in `goal account list`
    let alice = &accounts[2];
    let bob = &accounts[3];

    println!("addresses");
    println!("alice {}", alice.address);
    println!("bob {}", bob.address);
    println!("contract {:?}\n", contract_account.address);

    println!("starting balances");
    print_status(
        &algod,
        &alice.address.parse()?,
        &bob.address.parse()?,
        &contract_account.address,
    )
    .await;

    let params = algod.suggested_transaction_params().await.unwrap();

    let t = TxnBuilder::with(
        params,
        Pay::new(
            Address::from_str(&alice.address)?,
            contract_account.address,
            MicroAlgos(1_000_000),
        )
        .build(),
    )
    .build();

    // obtain a handle to our wallet
    let list_response = kmd.list_wallets().await?;
    let wallet_id = match list_response
        .wallets
        .into_iter()
        .find(|wallet| wallet.name == "unencrypted-default-wallet")
    {
        Some(wallet) => wallet.id,
        None => String::new(),
    };
    let init_response = kmd.init_wallet_handle(&wallet_id, "").await?;
    let wallet_handle_token = init_response.wallet_handle_token;

    // we need to sign the transaction to prove that we own the sender address
    let sign_response = kmd
        .sign_transaction(&wallet_handle_token, "", &t)
        .await
        .unwrap();

    kmd.release_wallet_handle(&wallet_handle_token).await?;

    // broadcast the transaction to the network
    let send_response = algod
        .broadcast_raw_transaction(&sign_response.signed_transaction)
        .await
        .unwrap();

    println!("alice->contract transaction id: {}\n", send_response.tx_id);

    wait_for_txn(&algod, &send_response.tx_id).await;

    println!("\nbalances after contract funded");
    print_status(
        &algod,
        &alice.address.parse()?,
        &bob.address.parse()?,
        &contract_account.address,
    )
    .await;

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
    let passphrase = "weather comfort erupt verb pet range endorse exhibit tree brush crane man";
    let passphrase_arg = passphrase.as_bytes().to_owned();

    let params = algod.suggested_transaction_params().await?;
    let t = TxnBuilder::with(
        params,
        Pay::new(
            contract_account.address,
            bob.address.parse()?,
            MicroAlgos(0),
        )
        .close_remainder_to(bob.address.parse()?)
        .build(),
    )
    .build();

    let signed_txn = contract_account.sign(&t, vec![passphrase_arg])?;
    let transaction_bytes = rmp_serde::to_vec_named(&signed_txn).unwrap();
    let send_response = algod
        .broadcast_raw_transaction(&transaction_bytes)
        .await
        .unwrap();

    wait_for_txn(&algod, &send_response.tx_id).await;

    println!("\nbalances after contract closed");
    print_status(
        &algod,
        &alice.address.parse()?,
        &bob.address.parse()?,
        &contract_account.address,
    )
    .await;

    Ok(())
}
