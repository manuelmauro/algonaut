//!
//! Implementation of the "Writing a Simple Smart Contract" tutorial
//! https://developer.algorand.org/tutorials/writing-simple-smart-contract/
//!
use algonaut::algod::v2::Algod;
use algonaut::algod::AlgodBuilder;
use algonaut::kmd::KmdBuilder;
use algonaut_core::{Address, MicroAlgos};
use algonaut_transaction::account::ContractAccount;
use algonaut_transaction::{Pay, TxnBuilder};
use dotenv::dotenv;
use std::env;
use std::error::Error;

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

        println!("txn {}... not confirmed; sleep 2s", &txn_id[..5]);
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}

async fn print_status(client: &Algod, alice: &Address, bob: &Address, contract: &Address) {
    println!(
        "\
alice    {}
bob      {}
contract {}
",
        get_balance(client, alice).await,
        get_balance(client, bob).await,
        get_balance(client, contract).await
    );
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

    let kmd = KmdBuilder::new()
        .bind(env::var("KMD_URL")?.as_ref())
        .auth(env::var("KMD_TOKEN")?.as_ref())
        .build_v1()?;

    // create contract account
    let compiled_teal = algod.compile_teal(TEAL_PROGRAM.as_bytes()).await?;
    let contract_account = ContractAccount::new(compiled_teal);

    // pick 2 accounts which show up in `goal account list`
    let alice = env::var("ALICE_ADDRESS")?;
    let bob = env::var("BOB_ADDRESS")?;

    println!("addresses");
    println!("alice    {}", alice);
    println!("bob      {}", bob);
    println!("contract {}\n", contract_account.address);

    println!("starting balances");
    print_status(
        &algod,
        &alice.parse()?,
        &bob.parse()?,
        &contract_account.address,
    )
    .await;

    // alice funds escrow
    let params = algod.suggested_transaction_params().await.unwrap();
    let t = TxnBuilder::with(
        params,
        Pay::new(
            alice.parse()?,
            contract_account.address,
            MicroAlgos(1_000_000),
        )
        .build(),
    )
    .build();

    // obtain a handle to our wallet and sign txn
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
    let sign_response = kmd
        .sign_transaction(&wallet_handle_token, "", &t)
        .await
        .unwrap();
    kmd.release_wallet_handle(&wallet_handle_token).await?;

    // submit transaction
    let send_response = algod
        .broadcast_raw_transaction(&sign_response.signed_transaction)
        .await
        .unwrap();

    println!("alice funds contract\n");

    wait_for_txn(&algod, &send_response.tx_id).await;

    println!("\nbalances after contract funded");
    print_status(
        &algod,
        &alice.parse()?,
        &bob.parse()?,
        &contract_account.address,
    )
    .await;

    // provide password to lsig and submit contract signed transaction
    let passphrase = "weather comfort erupt verb pet range endorse exhibit tree brush crane man";
    let passphrase_arg = passphrase.as_bytes().to_owned();

    let params = algod.suggested_transaction_params().await?;
    let t = TxnBuilder::with(
        params,
        Pay::new(contract_account.address, bob.parse()?, MicroAlgos(0))
            .close_remainder_to(bob.parse()?)
            .build(),
    )
    .build();

    let signed_txn = contract_account.sign(&t, vec![passphrase_arg])?;
    let transaction_bytes = rmp_serde::to_vec_named(&signed_txn).unwrap();
    let send_response = algod
        .broadcast_raw_transaction(&transaction_bytes)
        .await
        .unwrap();

    println!("send txn to close escrow\n");

    wait_for_txn(&algod, &send_response.tx_id).await;

    println!("\nbalances after contract closed");
    print_status(
        &algod,
        &alice.parse()?,
        &bob.parse()?,
        &contract_account.address,
    )
    .await;

    Ok(())
}
