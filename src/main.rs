use std::collections::BTreeMap;
use std::error::Error;
// use csv::{Reader, ReaderBuilder, Trim};
use serde::Deserialize;

type AcctMap = BTreeMap<u16, Acct>;

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

fn process_tx(tx: &Tx, accts: &mut AcctMap) { // TODO: will probably return Result
    let acct = accts.entry(tx.client_id).or_insert_with(Acct::default); // TODO: what if all tx's for a client are invalid?
    // TODO: validate transaction (ignore if missing tx ID and non-disputed tx ID)
    // TODO: match on tx.tx_type (process accordingly)
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

    /// A wrapper so I don't have to keep writing this in every test
    struct AcctMngr {
        accts: AcctMap,
        reader: Reader<&'static [u8]>,
    }
    impl AcctMngr {
        fn new(data: &'static [u8]) -> Self {
            Self {
                accts: AcctMap::new(),
                reader: ReaderBuilder::new().trim(Trim::All).from_reader(data),
            }
        }
    }

    #[test]
    fn deposits() {
        let mut mngr = AcctMngr::new(DEPOSITS.as_bytes());
        for res in mngr.reader.deserialize() {
            let tx: Tx = res.expect("unable to deserialize row");
            process_tx(&tx, &mut mngr.accts);  // <--- the function under test
        }
        assert!(mngr.accts.get(&1).is_some());
        assert!(mngr.accts.get(&2).is_some());
    }
}
