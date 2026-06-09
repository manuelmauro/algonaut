use crate::step_defs::unit::world::UnitWorld;
use algonaut_core::{Address, MICRO_ALGO_CONVERSION_FACTOR};
use algonaut_transaction::account::Account;
use cucumber::{given, then, when};
use std::error::Error;

// --- Encode / decode addresses --------------------------------------

#[when(expr = "I generate a key")]
async fn i_generate_a_key(w: &mut UnitWorld) {
    let account = Account::generate();
    w.account = Some(account.clone());
    w.address = Some(account.address());
}

#[when(expr = "I decode the address")]
async fn i_decode_the_address(w: &mut UnitWorld) -> Result<(), Box<dyn Error>> {
    let addr = w.address.expect("address not set");
    let s = addr.to_string();
    let parsed: Address = s.parse()?;
    w.address = Some(parsed);
    Ok(())
}

#[when(expr = "I encode the address")]
async fn i_encode_the_address(w: &mut UnitWorld) {
    // No-op: Address is already in canonical form; this step exists in
    // the feature to round-trip via the string representation, which
    // `I decode the address` already does.
    let _ = w.address;
}

#[then(expr = "the address should still be the same")]
async fn the_address_should_still_be_the_same(w: &mut UnitWorld) {
    let acc = w.account.as_ref().expect("account not set");
    let addr = w.address.expect("address not set");
    assert_eq!(addr, acc.address(), "address mismatch after roundtrip");
}

// --- Mnemonic <-> private key ---------------------------------------

#[given(regex = r#"^mnemonic for private key "([^"]+)"$"#)]
async fn mnemonic_for_private_key(
    w: &mut UnitWorld,
    mnemonic: String,
) -> Result<(), Box<dyn Error>> {
    w.account = Some(Account::from_mnemonic(&mnemonic)?);
    w.mnemonic = Some(mnemonic);
    Ok(())
}

#[when(expr = "I convert the private key back to a mnemonic")]
async fn convert_private_key_back_to_mnemonic(w: &mut UnitWorld) {
    let account = w.account.as_ref().expect("account not set");
    w.roundtrip_mnemonic = Some(account.mnemonic());
}

#[then(regex = r#"^the mnemonic should still be the same as "([^"]+)"$"#)]
async fn the_mnemonic_should_still_be_the_same_as(w: &mut UnitWorld, expected: String) {
    let actual = w
        .roundtrip_mnemonic
        .as_deref()
        .expect("round-tripped mnemonic not set");
    assert_eq!(actual, expected, "mnemonic roundtrip mismatch");
}

// --- Microalgos <-> Algos -------------------------------------------

#[when(regex = r"^I convert (\d+) microalgos to algos and back$")]
async fn convert_microalgos(w: &mut UnitWorld, micros: u64) {
    let algos = micros as f64 / MICRO_ALGO_CONVERSION_FACTOR;
    let back = (algos * MICRO_ALGO_CONVERSION_FACTOR).round() as u64;
    w.microalgos = Some(micros);
    w.roundtrip_microalgos = Some(back);
}

#[then(regex = r"^it should still be the same amount of microalgos (\d+)$")]
async fn it_should_still_be_the_same_amount_of_microalgos(w: &mut UnitWorld, expected: u64) {
    let original = w.microalgos.expect("microalgos not set");
    let back = w
        .roundtrip_microalgos
        .expect("roundtrip microalgos not set");
    assert_eq!(original, expected, "input mismatch");
    assert_eq!(back, expected, "microalgos roundtrip mismatch");
}
