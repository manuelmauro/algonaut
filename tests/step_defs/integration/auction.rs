use crate::step_defs::integration::world::World;
use algonaut_core::Address;
use algonaut_transaction::{
    account::Account,
    auction::{Bid, SignedBid},
};
use cucumber::{then, when};
use std::error::Error;

const BIDDER: &str = "DN7MBMCL5JQ3PFUQS7TMX5AH4EEKOBJVDUF4TCV6WERATKFLQF4MQUPZTA";

#[when(expr = "I create a bid")]
async fn i_create_a_bid(w: &mut World) -> Result<(), Box<dyn Error>> {
    let bidder: Address = BIDDER.parse()?;
    let bid = Bid {
        bidder_key: bidder,
        bid_currency: 1000,
        bid_id: 2,
        auction_id: 3,
        auction_key: bidder,
        max_price: 4,
    };
    w.bid = Some(bid);
    Ok(())
}

#[when(expr = "I sign the bid")]
async fn i_sign_the_bid(w: &mut World) -> Result<(), Box<dyn Error>> {
    let bid = w.bid.expect("bid not created");
    // We sign with a freshly-generated account; the test verifies the
    // signed-bid round-trips byte-for-byte, not that the signature
    // checks against any particular key.
    let account = Account::generate();
    w.signed_bid = Some(account.sign_bid(bid)?);
    Ok(())
}

#[when(expr = "I encode and decode the bid")]
async fn i_encode_and_decode_the_bid(w: &mut World) -> Result<(), Box<dyn Error>> {
    let signed = w.signed_bid.expect("signed bid not set");
    let bytes = rmp_serde::to_vec_named(&signed)?;
    let decoded: SignedBid = rmp_serde::from_slice(&bytes)?;
    w.signed_bid_roundtrip = Some(decoded);
    Ok(())
}

#[then(expr = "the bid should still be the same")]
async fn the_bid_should_still_be_the_same(w: &mut World) {
    let original = w.signed_bid.expect("signed bid not set");
    let decoded = w.signed_bid_roundtrip.expect("decoded signed bid not set");
    assert_eq!(original, decoded, "signed bid mismatch after roundtrip");
}
