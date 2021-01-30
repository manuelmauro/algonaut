use algorand_rs::Algod;

#[test]
fn test_proper_client_builder() -> Result<(), Box<dyn std::error::Error>> {
    let algod = Algod::new()
        .bind("http://localhost:4001")
        .auth("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        .client_v1();

    assert!(algod.ok().is_some());

    Ok(())
}
