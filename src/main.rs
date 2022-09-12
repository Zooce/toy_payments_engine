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

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum TxType {
    Deposit,
    Withdraw,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize, PartialEq)]
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

    struct TestDef {
        input_data: &'static str,
        expected_transactions: Vec<(u32, Tx)>,
        expected_accounts: Vec<(u16, Acct)>,
    }

    impl TestDef {
        fn run(&self) -> Result<(), Box<dyn Error>> {
            // engine to do the processing
            let mut engine = Engine::default();

            // build a reader for the csv data
            let mut reader = ReaderBuilder::new().trim(Trim::All).from_reader(self.input_data.as_bytes());

            // do the processing
            let mut ps_res = Ok(());
            for res in reader.deserialize() {
                let tx: Tx = res.expect("unable to deserialize row");
                if let Err(e) = engine.process_tx(tx) {
                    ps_res = Err(e);
                    break;
                }
            };

            // verify transactions
            assert_eq!(self.expected_transactions.len(), engine.tx_map.len());
            for (id, tx) in &self.expected_transactions {
                let t = engine.tx_map.get(&id).expect("expected transaction {id}");
                assert_eq!(*tx, *t);
            }

            // verify accounts
            assert_eq!(self.expected_accounts.len(), engine.acct_map.len());
            for (id, acct) in &self.expected_accounts {
                let a = engine.acct_map.get(&id).expect("expected account for client {id}");
                assert_eq!(*acct, *a);
            }

            ps_res
        }
    }

    #[test]
    fn deposits() {
        let test = TestDef{
            input_data: "type, client, tx, amount
                deposit,    1,  1,  1.0
                deposit,    2,  2,  2.0
                deposit,    1,  3,  2.0",
            expected_transactions: vec![
                (1, Tx{ tx_type: TxType::Deposit, client_id: 1, tx_id: 1, amount: Some(1.0) }),
                (2, Tx{ tx_type: TxType::Deposit, client_id: 2, tx_id: 2, amount: Some(2.0) }),
                (3, Tx{ tx_type: TxType::Deposit, client_id: 1, tx_id: 3, amount: Some(2.0) }),
            ],
            expected_accounts: vec![
                (1, Acct{ available: 3.0, held: 0.0, total: 3.0, locked: false }),
                (2, Acct{ available: 2.0, held: 0.0, total: 2.0, locked: false }),
            ],
        };
        assert!(test.run().is_ok());
    }

    #[test]
    fn withdraws() {
        let test = TestDef{
            input_data: "type, client, tx, amount
                deposit,    1,  1,  1.0
                deposit,    2,  2,  2.0
                withdraw,   1,  3,  0.5",
            expected_transactions: vec![
                (1, Tx{ tx_type: TxType::Deposit, client_id: 1, tx_id: 1, amount: Some(1.0) }),
                (2, Tx{ tx_type: TxType::Deposit, client_id: 2, tx_id: 2, amount: Some(2.0) }),
                (3, Tx{ tx_type: TxType::Withdraw, client_id: 1, tx_id: 3, amount: Some(0.5) }),
            ],
            expected_accounts: vec![
                (1, Acct{ available: 0.5, held: 0.0, total: 0.5, locked: false }),
                (2, Acct{ available: 2.0, held: 0.0, total: 2.0, locked: false }),
            ],
        };
        assert!(test.run().is_ok());
    }

    #[test]
    fn withdraw_error() {
        let test = TestDef{
            input_data: "type, client, tx, amount
                deposit,    1,  1,  1.0
                deposit,    2,  2,  2.0
                withdraw,   1,  3,  1.1",
            expected_transactions: vec![
                (1, Tx{ tx_type: TxType::Deposit, client_id: 1, tx_id: 1, amount: Some(1.0) }),
                (2, Tx{ tx_type: TxType::Deposit, client_id: 2, tx_id: 2, amount: Some(2.0) }),
            ],
            expected_accounts: vec![
                (1, Acct{ available: 1.0, held: 0.0, total: 1.0, locked: false }),
                (2, Acct{ available: 2.0, held: 0.0, total: 2.0, locked: false }),
            ],
        };
        assert!(test.run().is_err());
    }
}
