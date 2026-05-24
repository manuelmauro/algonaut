//! Integration tests for the `contract!` macro.

use algonaut::contract;
use algonaut_core::AppId;
use algonaut_transaction::account::Account;
use std::sync::Arc;

// Generate the Calculator contract client
contract!("tests/fixtures/calculator.json");

#[test]
fn test_generated_struct_exists() {
    let alice = Account::generate();
    let address = alice.address();
    let signer = Arc::new(alice);

    // Test that the generated struct can be created
    let calculator = Calculator::new(AppId(123), address, signer);

    // Test that accessor methods work
    assert_eq!(calculator.app_id(), AppId(123));
    assert_eq!(calculator.sender(), address);
}

#[test]
fn test_network_constructors() {
    use algonaut_transaction::Signer;

    let alice = Account::generate();
    let address = alice.address();
    let signer: Arc<dyn Signer> = Arc::new(alice);

    // Test testnet constructor (app ID 123 from the JSON)
    let testnet_calculator = Calculator::testnet(address, Arc::clone(&signer));
    assert_eq!(testnet_calculator.app_id(), AppId(123));

    // Test mainnet constructor (app ID 456 from the JSON)
    let mainnet_calculator = Calculator::mainnet(address, signer);
    assert_eq!(mainnet_calculator.app_id(), AppId(456));
}

#[test]
fn test_method_builders_exist() {
    let alice = Account::generate();
    let address = alice.address();
    let signer = Arc::new(alice);
    let calculator = Calculator::new(AppId(123), address, signer);

    // Test that method builders can be created
    // These calls don't actually build or execute, just verify the methods exist
    // and accept the correct argument types
    let _add_builder = calculator.add(2u64, 3u64);
    let _subtract_builder = calculator.subtract(5u64, 2u64);
    let _multiply_builder = calculator.multiply(4u64, 5u64);
    let _divide_builder = calculator.divide(10u64, 2u64);
    let _bool_builder = calculator.echo_bool(true);
    let _string_builder = calculator.echo_string("hello".to_string());
    let _bytes_builder = calculator.echo_bytes(vec![1, 2, 3]);
    let _address_builder = calculator.echo_address(address);
    let _noop_builder = calculator.noop();
}

#[test]
fn test_method_builder_returns_method_call() {
    use algonaut_crypto::HashDigest;
    use algonaut_model::algod::SuggestedParams;

    let alice = Account::generate();
    let address = alice.address();
    let signer = Arc::new(alice);
    let calculator = Calculator::new(AppId(123), address, signer);

    // Create mock suggested params
    let params = SuggestedParams {
        consensus_version: "test".to_string(),
        fee: algonaut_core::MicroAlgos(0),
        genesis_hash: HashDigest([0u8; 32]),
        genesis_id: "test".to_string(),
        last_round: algonaut_core::Round(1000),
        min_fee: algonaut_core::MicroAlgos(1000),
    };

    // Build a method call
    let method_call = calculator.add(2u64, 3u64).build(&params);

    // Verify the method call is configured correctly
    // (We can't easily inspect the internals, but at least we verify it compiles)
    let _ = method_call;
}
