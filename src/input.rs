use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TxType {
    Deposit,
    Withdraw,
    Dispute,
    Resolve,
    Chargeback,
}

/// This type represents a row in the input CSV.
#[derive(Debug, Deserialize)]
pub struct Tx {
    /// The type of this transaction.
    #[serde(rename = "type")]
    pub tx_type: TxType,

    /// The client ID associated with this transaction.
    #[serde(rename = "client")]
    pub client_id: u16,

    /// The transaction ID of this transaction or a referenced transaction.
    #[serde(rename = "tx")]
    pub tx_id: u32,

    /// The amount of this transaction - required for deposits and withdraws.
    pub amount: Option<f64>,
}
