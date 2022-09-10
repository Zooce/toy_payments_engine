use std::error::Error;
use csv::Reader;
use serde::Deserialize;

fn main() -> Result<(), Box<dyn Error>> {
    let txs = "type,client,tx,amount\n\
        deposit,1,1,1.0\n\
        deposit,2,2,2.0\n\
        deposit,1,3,2.0\n\
        withdraw,1,4,1.5\n\
        withdraw,2,5,3.0\n\
        dispute,1,3,\n\
        dispute,1,4,\n\
        resolve,1,3,\n\
        chargeback,1,4,";
    // TODO: open file from first argument
    let mut reader = Reader::from_reader(txs.as_bytes());
    for result in reader.deserialize() {
        let tx: Tx = result?;
        println!("{tx:?}");
        // TODO: process record/transaction
    }
    // TODO: write all accounts to std::io::stdout()
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TxType {
    Deposit,
    Withdraw,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize)]
struct Tx {
    #[serde(rename = "type")]
    tx_type: TxType,
    #[serde(rename = "client")]
    client_id: u16,
    #[serde(rename = "tx")]
    tx_id: u32,
    amount: Option<f64>,
}
