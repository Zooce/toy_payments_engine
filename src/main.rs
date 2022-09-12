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
        let acct = self.acct_map.entry(tx.client_id).or_insert_with(Acct::default); // TODO: what if all tx's for a client are invalid?
        // TODO: validate transaction (ignore if missing tx ID and non-disputed tx ID)
        match tx.tx_type {
            TxType::Deposit => acct.deposit(tx.amount.expect("deposit transactions must have an amount")),
            TxType::Withdraw => acct.withdraw(tx.amount.expect("withdraw transactions must have an amount"))?,
            _ => todo!(),
        }
        _ = self.tx_map.entry(tx.tx_id).or_insert(tx); // TODO: dispute-related transactions do not get stored here, rather they modify transactions
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

#[derive(Debug, Default, PartialEq)]
struct Acct {
    available: f64,
    held: f64,
    total: f64,
    locked: bool,
}

impl Acct {
    fn deposit(&mut self, amt: f64) {
        self.total += amt;
        self.available += amt;
    }

    fn withdraw(&mut self, amt: f64) -> Result<(), &'static str> { // TODO: use a better error type
        if self.available < amt {
            return Err("funds not available for withdraw");
        }
        self.total -= amt;
        self.available -= amt;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use csv::{ReaderBuilder, Trim};

    fn process_txs(engine: &mut Engine, data: &[u8]) -> Result<(), Box<dyn Error>> {
        let mut reader = ReaderBuilder::new().trim(Trim::All).from_reader(data);
        for res in reader.deserialize() {
            let tx: Tx = res.expect("unable to deserialize row");
            engine.process_tx(tx)?
        }
        Ok(())
    }

    #[test]
    fn deposits() {
        const DATA: &str =
            "type, client, tx, amount
            deposit,    1,  1,  1.0
            deposit,    2,  2,  2.0
            deposit,    1,  3,  2.0";
        const A1: Acct = Acct{
            available: 3.0,
            held: 0.0,
            total: 3.0,
            locked: false,
        };
        const A2: Acct = Acct{
            available: 2.0,
            held: 0.0,
            total: 2.0,
            locked: false,
        };

        let mut engine = Engine::default();
        process_txs(&mut engine, DATA.as_bytes()).expect("transaction processing failed");

        // counts
        assert_eq!(3, engine.tx_map.len());
        assert_eq!(2, engine.acct_map.len());

        // account ids
        let a1 = engine.acct_map.get(&1).expect("expected account for client {1}");
        let a2 = engine.acct_map.get(&2).expect("expected account for client {2}");

        // account funds
        assert_eq!(A1, *a1);
        assert_eq!(A2, *a2);
    }

    #[test]
    fn withdraws() {
        const DATA: &str =
            "type, client, tx, amount
            deposit,    1,  1,  1.0
            deposit,    2,  2,  2.0
            withdraw,   1,  3,  0.5";
        const A1: Acct = Acct{
            available: 0.5,
            held: 0.0,
            total: 0.5,
            locked: false,
        };
        const A2: Acct = Acct{
            available: 2.0,
            held: 0.0,
            total: 2.0,
            locked: false,
        };

        let mut engine = Engine::default();
        process_txs(&mut engine, DATA.as_bytes()).expect("transaction processing failed");

        // counts
        assert_eq!(3, engine.tx_map.len());
        assert_eq!(2, engine.acct_map.len());

        // account ids
        let a1 = engine.acct_map.get(&1).expect("expected account for client {1}");
        let a2 = engine.acct_map.get(&2).expect("expected account for client {2}");

        // account funds
        assert_eq!(A1, *a1);
        assert_eq!(A2, *a2);
    }

    #[test]
    fn withdraw_error() {
        const DATA: &str =
            "type, client, tx, amount
            deposit,    1,  1,  1.0
            deposit,    2,  2,  2.0
            withdraw,   1,  3,  1.1";
        const A1: Acct = Acct{
            available: 1.0,
            held: 0.0,
            total: 1.0,
            locked: false,
        };
        const A2: Acct = Acct{
            available: 2.0,
            held: 0.0,
            total: 2.0,
            locked: false,
        };

        let mut engine = Engine::default();
        assert!(process_txs(&mut engine, DATA.as_bytes()).is_err());

        // counts
        assert_eq!(2, engine.tx_map.len());
        assert_eq!(2, engine.acct_map.len());

        // account ids
        let a1 = engine.acct_map.get(&1).expect("expected account for client {1}");
        let a2 = engine.acct_map.get(&2).expect("expected account for client {2}");

        // account funds
        assert_eq!(A1, *a1);
        assert_eq!(A2, *a2);
    }
}
