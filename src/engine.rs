use std::collections::BTreeMap;

use crate::account::Acct;
use crate::transaction::{Tx, TxType};

/// Represents the current state (in terms of disputes) of a recorded transaction.
#[derive(Debug, PartialEq)]
pub enum TxState {
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
pub struct RecTx {
    pub client_id: u16,
    pub amount: f64,
    pub state: TxState,
}

impl From<Tx> for RecTx {
    fn from(tx: Tx) -> Self {
        Self {
            client_id: tx.client_id,
            amount: match tx.tx_type {
                TxType::Deposit => tx.amount.unwrap(),
                TxType::Withdrawal => -tx.amount.unwrap(),
                _ => unreachable!(),
            },
            state: TxState::Undisputed
        }
    }
}

/// The map of transactions - needed so that past transactions can be disputed
type TxMap = BTreeMap<u32, RecTx>;
/// The map of accounts - this is the output of the program
type AcctMap = BTreeMap<u16, Acct>;

#[derive(Default)]
pub struct Engine {
    /// Keeps track of all transactions processed by the engine
    pub tx_map: TxMap,
    /// Keeps track of all client accounts
    pub acct_map: AcctMap,
}

impl Engine {
    pub fn process_tx(&mut self, tx: Tx) -> Result<(), String> {
        // 1. Get the account associated with this transaction
        // NOTE: even if all transactions for an account are invalid we create a default account
        let acct = self.acct_map.entry(tx.client_id).or_insert_with(Acct::default);

        // 2. Locked accounts are locked forever - no transactions can be processed for them
        if acct.locked {
            return Err(format!("unable to process transaction - account locked"));
        }

        // 3a. Process "recorded" transactions (i.e. deposits and withdraws)
        if let TxType::Deposit | TxType::Withdrawal = tx.tx_type {
            if self.tx_map.contains_key(&tx.tx_id) {
                return Err(format!("transaction id {} already exists", tx.tx_id));
            }
            match tx.amount {
                Some(amt) => {
                    match &tx.tx_type {
                        TxType::Deposit => acct.deposit(amt)?,
                        TxType::Withdrawal => acct.withdrawal(amt)?,
                        _ => unreachable!(),
                    }
                    self.tx_map.insert(tx.tx_id, tx.into());
                }
                None => return Err(format!("transaction {} missing amount", tx.tx_id)),
            }
        }

        // 3b. Process "non-recorded" transaction (i.e. dispute-related)
        // NOTE: all dispute-related transactions only make sense if their transaction ID exists
        else if let Some(mut t) = self.tx_map.get_mut(&tx.tx_id) {
            match &tx.tx_type {
                TxType::Deposit | TxType::Withdrawal => unreachable!(),
                TxType::Dispute if TxState::Undisputed == t.state => {
                    t.state = TxState::Disputed;
                    acct.dispute(t.amount);
                }
                TxType::Resolve if TxState::Disputed == t.state => {
                    t.state = TxState::Undisputed;
                    acct.resolve(t.amount);
                }
                TxType::Chargeback if TxState::Disputed == t.state => {
                    t.state = TxState::Chargebacked;
                    acct.chargeback(t.amount);
                }
                _ => return Err(format!("invalid tx {:?} for state {:?}", tx.tx_type, t.state)),
            }
        }
        Ok(())
    }
}

//------------------------------------------------------------------------------
//------------------------------------------------------------------------------
// TESTS
//------------------------------------------------------------------------------
//------------------------------------------------------------------------------

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
        fn run(&self) -> Result<(), String> {
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
                withdrawal, 1,  3,  0.5",
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
                withdrawal, 1,  3,  1.1",
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
                withdrawal, 1,  2,  0.5
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
                withdrawal, 1,  2,  0.5
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
                withdrawal, 1,  2,  0.5
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
