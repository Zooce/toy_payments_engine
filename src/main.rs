use std::collections::BTreeMap;
use std::error::Error;
// use csv::{Reader, ReaderBuilder, Trim};
use serde::Deserialize;

#[derive(Debug)]
enum EngineError {
    TxAlreadyDisputed,
    TxNotDisputed,
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{self:?}"))
    }
}

impl Error for EngineError {} // take the default implementations for now

/// The map of transactions - needed so that past transactions can be disputed
type TxMap = BTreeMap<u32, RecTx>;
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
        // TODO: validate transaction (valid tx ID for non-recorded transactions, given amount for recorded transactions, locked account)
        match tx.tx_type {
            TxType::Deposit => acct.deposit(tx.amount.expect("deposit transactions must have an amount")),
            TxType::Withdraw => acct.withdraw(tx.amount.expect("withdraw transactions must have an amount"))?,

            // Dispute-related Assumptions:
            //  - If a dispute-related transaction references a non-existent deposit/withdraw, we count it as an error
            //    in the input and simply ignore it.
            //  - Any scenario that does not fit the dispute-resolve-chargeback state machine is counted as an error in
            //    the input and ignored (see disputes.excalidraw.png at the root of this repository).

            // TODO: consolidate the common code here
            TxType::Dispute => if let Some(mut t) = self.tx_map.get_mut(&tx.tx_id) {
                match t.state {
                    TxState::Undisputed => t.state = TxState::Disputed,
                    _ => return Err(Box::new(EngineError::TxAlreadyDisputed)),
                }
                acct.dispute(t.amount);
            }
            TxType::Resolve => if let Some(mut t) = self.tx_map.get_mut(&tx.tx_id) {
                match t.state {
                    TxState::Disputed => t.state = TxState::Undisputed,
                    _ => return Err(Box::new(EngineError::TxNotDisputed)),
                }
                acct.resolve(t.amount);
            }
            TxType::Chargeback => if let Some(mut t) = self.tx_map.get_mut(&tx.tx_id) {
                match t.state {
                    TxState::Disputed => t.state = TxState::Chargebacked,
                    _ => return Err(Box::new(EngineError::TxNotDisputed)),
                }
                acct.chargeback(t.amount);
            }
        }
        // only deposits and withdraws are recorded (other types only reference these)
        if let TxType::Deposit | TxType::Withdraw = tx.tx_type {
            _ = self.tx_map.entry(tx.tx_id).or_insert(tx.into());
        }
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

/// This type represents a row in the input CSV.
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

/// Represents the current state (in terms of disputes) of a recorded transaction.
#[derive(Debug, PartialEq)]
enum TxState {
    /// The transaction is okay.
    Undisputed,
    /// The transaction is currently being disputed,
    Disputed,
    /// The transaction was successfully disputed.
    Chargebacked,
}

/// A recorded transaction is different from `Tx` in that these only represent
/// transactions with amounts (i.e., deposits and withdraws).
#[derive(Debug, PartialEq)]
struct RecTx {
    client_id: u16,
    amount: f64,
    state: TxState,
}

impl From<Tx> for RecTx {
    fn from(tx: Tx) -> Self {
        Self {
            client_id: tx.client_id,
            amount: match tx.tx_type {
                TxType::Deposit => tx.amount.unwrap(),
                TxType::Withdraw => -tx.amount.unwrap(),
                _ => unreachable!(),
            },
            state: TxState::Undisputed
        }
    }
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

    fn dispute(&mut self, amt: f64) {
        // TODO: what if this account does not have `amt` available for the dispute?
        self.available -= amt;
        self.held += amt;
    }

    fn resolve(&mut self, amt: f64) {
        self.available += amt;
        self.held -= amt;
    }

    fn chargeback(&mut self, amt: f64) {
        self.held -= amt;
        self.total -= amt;
        self.locked = true;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use csv::{ReaderBuilder, Trim};

    struct TestDef {
        input_data: &'static str,
        expected_transactions: Vec<(u32, RecTx)>,
        expected_accounts: Vec<(u16, Acct)>,
    }

    impl TestDef {
        fn run(&self) -> Result<(), Box<dyn Error>> {
            // engine to do the processing
            let mut engine = Engine::default();

            // build a reader for the csv data
            let mut reader = ReaderBuilder::new()
                .trim(Trim::All)
                .flexible(true)
                .from_reader(self.input_data.as_bytes());

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
            input_data:
                "type, client, tx, amount
                deposit,    1,  1,  1.0
                deposit,    2,  2,  2.0
                deposit,    1,  3,  2.0",
            expected_transactions: vec![
                (1, RecTx{ client_id: 1, amount: 1.0, state: TxState::Undisputed }),
                (2, RecTx{ client_id: 2, amount: 2.0, state: TxState::Undisputed }),
                (3, RecTx{ client_id: 1, amount: 2.0, state: TxState::Undisputed }),
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
            input_data:
                "type, client, tx, amount
                deposit,    1,  1,  1.0
                deposit,    2,  2,  2.0
                withdraw,   1,  3,  0.5",
            expected_transactions: vec![
                (1, RecTx{ client_id: 1, amount: 1.0, state: TxState::Undisputed }),
                (2, RecTx{ client_id: 2, amount: 2.0, state: TxState::Undisputed }),
                (3, RecTx{ client_id: 1, amount: -0.5, state: TxState::Undisputed }),
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
            input_data:
                "type, client, tx, amount
                deposit,    1,  1,  1.0
                deposit,    2,  2,  2.0
                withdraw,   1,  3,  1.1",
            expected_transactions: vec![
                (1, RecTx{ client_id: 1, amount: 1.0, state: TxState::Undisputed }),
                (2, RecTx{ client_id: 2, amount: 2.0, state: TxState::Undisputed }),
            ],
            expected_accounts: vec![
                (1, Acct{ available: 1.0, held: 0.0, total: 1.0, locked: false }),
                (2, Acct{ available: 2.0, held: 0.0, total: 2.0, locked: false }),
            ],
        };
        assert!(test.run().is_err());
    }

    #[test]
    fn dispute_deposit() {
        let test = TestDef{
            input_data:
                "type, client, tx, amount
                deposit,    1,  1,  1.0
                deposit,    2,  2,  2.0
                dispute,    1,  1,  ",      // NOTE - we can't end the CSV data with a newline when the last line has a blank optional value
            expected_transactions: vec![
                (1, RecTx{ client_id: 1, amount: 1.0, state: TxState::Disputed }),
                (2, RecTx{ client_id: 2, amount: 2.0, state: TxState::Undisputed }),
            ],
            expected_accounts: vec![
                (1, Acct{ available: 0.0, held: 1.0, total: 1.0, locked: false }),
                (2, Acct{ available: 2.0, held: 0.0, total: 2.0, locked: false }),
            ],
        };
        assert!(test.run().is_ok());
    }

    #[test]
    fn dispute_withdraw() {
        let test = TestDef{
            input_data:
                "type, client, tx, amount
                deposit,    1,  1,  1.0
                withdraw,   1,  2,  0.5
                dispute,    1,  2,  ",      // NOTE - we can't end the CSV data with a newline when the last line has a blank optional value
            expected_transactions: vec![
                (1, RecTx{ client_id: 1, amount: 1.0, state: TxState::Undisputed }),
                (2, RecTx{ client_id: 1, amount: -0.5, state: TxState::Disputed }),
            ],
            expected_accounts: vec![
                (1, Acct{ available: 1.0, held: -0.5, total: 0.5, locked: false }),
            ],
        };
        assert!(test.run().is_ok());
    }

    #[test]
    fn resolve_deposit() {
        let test = TestDef{
            input_data:
                "type, client, tx, amount
                deposit,    1,  1,  1.0
                deposit,    2,  2,  2.0
                dispute,    1,  1,
                resolve,    1,  1,  ",      // NOTE - we can't end the CSV data with a newline when the last line has a blank optional value
            expected_transactions: vec![
                (1, RecTx{ client_id: 1, amount: 1.0, state: TxState::Undisputed }),
                (2, RecTx{ client_id: 2, amount: 2.0, state: TxState::Undisputed }),
            ],
            expected_accounts: vec![
                (1, Acct{ available: 1.0, held: 0.0, total: 1.0, locked: false }),
                (2, Acct{ available: 2.0, held: 0.0, total: 2.0, locked: false }),
            ],
        };
        assert!(test.run().is_ok());
    }

    #[test]
    fn resolve_withdraw() {
        let test = TestDef{
            input_data:
                "type, client, tx, amount
                deposit,    1,  1,  1.0
                withdraw,   1,  2,  0.5
                dispute,    1,  2,
                resolve,    1,  2,  ",      // NOTE - we can't end the CSV data with a newline when the last line has a blank optional value
            expected_transactions: vec![
                (1, RecTx{ client_id: 1, amount: 1.0, state: TxState::Undisputed }),
                (2, RecTx{ client_id: 1, amount: -0.5, state: TxState::Undisputed }),
            ],
            expected_accounts: vec![
                (1, Acct{ available: 0.5, held: 0.0, total: 0.5, locked: false }),
            ],
        };
        assert!(test.run().is_ok());
    }

    #[test]
    fn chargeback_deposit() {
        let test = TestDef{
            input_data:
                "type, client, tx, amount
                deposit,    1,  1,  1.0
                deposit,    2,  2,  2.0
                dispute,    1,  1,
                chargeback, 1,  1,  ",      // NOTE - we can't end the CSV data with a newline when the last line has a blank optional value
            expected_transactions: vec![
                (1, RecTx{ client_id: 1, amount: 1.0, state: TxState::Chargebacked }),
                (2, RecTx{ client_id: 2, amount: 2.0, state: TxState::Undisputed }),
            ],
            expected_accounts: vec![
                (1, Acct{ available: 0.0, held: 0.0, total: 0.0, locked: true }),
                (2, Acct{ available: 2.0, held: 0.0, total: 2.0, locked: false }),
            ],
        };
        assert!(test.run().is_ok());
    }

    #[test]
    fn chargeback_withdraw() {
        let test = TestDef{
            input_data:
                "type, client, tx, amount
                deposit,    1,  1,  1.0
                withdraw,   1,  2,  0.5
                dispute,    1,  2,
                chargeback, 1,  2,  ",      // NOTE - we can't end the CSV data with a newline when the last line has a blank optional value
            expected_transactions: vec![
                (1, RecTx{ client_id: 1, amount: 1.0, state: TxState::Undisputed }),
                (2, RecTx{ client_id: 1, amount: -0.5, state: TxState::Chargebacked }),
            ],
            expected_accounts: vec![
                (1, Acct{ available: 1.0, held: 0.0, total: 1.0, locked: true }),
            ],
        };
        assert!(test.run().is_ok());
    }

    // TODO : test input error scenarios
}
