//! Algorand protocol domain separators (hash prefixes).
//!
//! These byte-string prefixes are prepended to data before hashing to ensure
//! domain separation across different contexts (transactions, groups, bids, etc.).

/// Prefix for signing arbitrary messages ("MX").
pub const MESSAGE_PREFIX: &[u8] = b"MX";

/// Prefix for transaction hashes ("TX").
pub const TRANSACTION_PREFIX: &[u8] = b"TX";

/// Prefix for transaction group hashes ("TG").
pub const TRANSACTION_GROUP_PREFIX: &[u8] = b"TG";

/// Prefix for multisig address derivation ("MultisigAddr").
pub const MULTISIG_ADDR_PREFIX: &[u8] = b"MultisigAddr";

/// Prefix for program (TEAL) hashes ("Program").
pub const PROGRAM_PREFIX: &[u8] = b"Program";

/// Prefix for bid signatures ("aB").
pub const BID_PREFIX: &[u8] = b"aB";
