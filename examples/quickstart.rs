use algosdk::{AlgodClient, KmdClient};

fn main() {
    let algod_address = "http://localhost:8080";
    let algod_token = "contents-of-algod.token";
    let kmd_address = "http://localhost:7833";
    let kmd_token = "contents-of-kmd.token";

    let algod_client = AlgodClient::new(algod_address, algod_token);
    let kmd_client = KmdClient::new(kmd_address, kmd_token);

    println!(
        "Algod versions: {:?}",
        algod_client.versions().unwrap().versions
    );
    println!(
        "Kmd versions: {:?}",
        kmd_client.versions().unwrap().versions
    );
}
