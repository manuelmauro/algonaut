/*
 * Indexer
 *
 * Algorand ledger analytics API.
 *
 * The version of the OpenAPI document: 2.0
 *
 * Generated by: https://openapi-generator.tech
 */

/// TransactionPayment : Fields for a payment transaction.  Definition: data/transactions/payment.go : PaymentTxnFields

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct TransactionPayment {
    /// \\[amt\\] number of MicroAlgos intended to be transferred.
    #[serde(rename = "amount")]
    pub amount: u64,
    /// Number of MicroAlgos that were sent to the close-remainder-to address when closing the sender account.
    #[serde(rename = "close-amount", skip_serializing_if = "Option::is_none")]
    pub close_amount: Option<u64>,
    /// \\[close\\] when set, indicates that the sending account should be closed and all remaining funds be transferred to this address.
    #[serde(rename = "close-remainder-to", skip_serializing_if = "Option::is_none")]
    pub close_remainder_to: Option<String>,
    /// \\[rcv\\] receiver's address.
    #[serde(rename = "receiver")]
    pub receiver: String,
}

impl TransactionPayment {
    /// Fields for a payment transaction.  Definition: data/transactions/payment.go : PaymentTxnFields
    pub fn new(amount: u64, receiver: String) -> TransactionPayment {
        TransactionPayment {
            amount,
            close_amount: None,
            close_remainder_to: None,
            receiver,
        }
    }
}
