use std::error::Error;
use csv::Reader;

fn main() -> Result<(), Box<dyn Error>> {
    let txs = "type,client,tx,amount\n\
        deposit,1,1,1.0\n\
        deposit,2,2,2.0\n\
        deposit,1,3,2.0\n\
        withdraw,1,4,1.5\n\
        withdraw,2,5,3.0";
    // TODO: open file from first argument
    let mut reader = Reader::from_reader(txs.as_bytes());
    for result in reader.records() {
        let tx = result?;
        println!("{tx:?}");
        // TODO: process record/transaction
    }
    // TODO: write all accounts to std::io::stdout()
    Ok(())
}
