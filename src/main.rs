use std::collections::BTreeMap;
use std::error::Error;
// use csv::{Reader, ReaderBuilder, Trim};
use serde::Deserialize;

/// The map of transactions - needed so that past transactions can be disputed
type TxMap = BTreeMap<u32, Tx>;  // TODO: we really only need to store the client ID, the amount (Deposit = +amount, Withdraw = -amount), and whether it's currently disputed
/// The map of accounts - this is the output of the program
type AcctMap = BTreeMap<u16, Acct>;

#[derive(Default)]
struct Engine {
    /// Keeps track of all transactions processed by the engine
    tx_map: TxMap,
    /// Keeps track of all client accounts
    acct_map: AcctMap,
}

impl Engine {
    fn process_tx(&mut self, tx: Tx) -> Result<(), Box<dyn Error>> {
        let _acct = self.acct_map.entry(tx.client_id).or_insert_with(Acct::default); // TODO: what if all tx's for a client are invalid?
        _ = self.tx_map.entry(tx.tx_id).or_insert(tx); // TODO: dispute-related transactions do not get stored here, rather they modify transactions
        // TODO: validate transaction (ignore if missing tx ID and non-disputed tx ID)
        // TODO: match on tx.tx_type (process accordingly)
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
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

#[derive(Debug, Default)]
struct Acct {
    available: f64,
    held: f64,
    total: f64,
    locked: bool,
}

#[cfg(test)]
mod test {
    use super::*;
    use csv::{Reader, ReaderBuilder, Trim};

    const DEPOSITS: &str =
        "type, client, tx, amount
        deposit,    1,  1,  1.0
        deposit,    2,  2,  2.0
        deposit,    1,  3,  2.0";

    fn reader(data: &'static [u8]) -> Reader<&'static [u8]> {
        ReaderBuilder::new().trim(Trim::All).from_reader(data)
    }

    #[test]
    fn deposits() {
        let mut engine = Engine::default();
        for res in reader(DEPOSITS.as_bytes()).deserialize() {
            let tx: Tx = res.expect("unable to deserialize row");
            engine.process_tx(tx).expect("filed to process transaction");  // <--- the function under test
        }

        // counts
        assert_eq!(3, engine.tx_map.len());
        assert_eq!(2, engine.acct_map.len());

        // account ids
        let _a1 = engine.acct_map.get(&1).expect("expected account for client {1}");
        let _a2 = engine.acct_map.get(&2).expect("expected account for client {2}");
    }
}
