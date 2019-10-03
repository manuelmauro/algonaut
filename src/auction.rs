use serde::{Deserialize, Serialize};

use crate::crypto::{Address, Signature};

/// A bid by a user as part of an auction.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Bid {
    /// Identifies the auction for which this bid is intended
    #[serde(rename = "aid")]
    pub auction_id: u64,
    /// The auction for this bid.
    #[serde(rename = "auc")]
    pub auction_key: Address,
    /// The bidder placing this bid.
    #[serde(rename = "bidder")]
    pub bidder_key: Address,
    /// How much external currency the bidder is putting in with this bid.
    #[serde(rename = "cur")]
    pub bid_currency: u64,
    /// Identifies this bid. The first bid by a bidder with a particular bid_id on the blockchain will be
    /// considered, preventing replay of bids. Specifying a different bid_id allows the bidder to place multiple bids in an auction.
    #[serde(rename = "id")]
    pub bid_id: u64,
    /// The maximum price, in units of external currency per Algo, that the bidder is willing to pay.
    /// This must be as high as the current price of the auction in the block in which this bid appears.
    #[serde(rename = "price")]
    pub max_price: u64,
}

/// A signed bid by a bidder.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SignedBid {
    /// Bid contains information about the bid.
    pub bid: Bid,
    /// A signature by the bidder, as identified in the bid ([Bid.bidder_key]) over the hash of the Bid.
    pub sig: Signature,
}
