use algorust::algod::AlgodClient;

#[test]
fn test_client_status() {
    let algod_address = "http://localhost:4001";
    let algod_token = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    let algod_client = AlgodClient::new(algod_address, algod_token);

    assert!(algod_client.status().is_ok());
}
