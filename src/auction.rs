use serde::{Deserialize, Serialize};

use crate::crypto::{Address, Signature};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Bid {
    #[serde(rename = "aid")]
    pub auction_id: u64,
    #[serde(rename = "auc")]
    pub auction_key: Address,
    #[serde(rename = "bidder")]
    pub bidder_key: Address,
    #[serde(rename = "cur")]
    pub bid_currency: u64,
    #[serde(rename = "id")]
    pub bid_id: u64,
    #[serde(rename = "price")]
    pub max_price: u64,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SignedBid {
    pub bid: Bid,
    pub sig: Signature,
}
