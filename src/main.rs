use std::error::Error;
use std::fs::File;
use std::io::stdout;

mod account;
mod engine;
mod input;
mod output;
mod transaction;

// NOTE: The `csv` crate related code is mostly taken from its documentation.

fn main() -> Result<(), Box<dyn Error>> {
    let mut engine = engine::Engine::default();

    let file_path = input::get_first_arg()?;
    let file = File::open(file_path)?;
    let mut reader = input::reader(file);

    for result in reader.deserialize() {
        let tx: transaction::Tx = result?;
        _ = engine.process_tx(tx); // purposefully ignoring errors for now
    }

    let mut writer = output::writer(stdout());
    writer.write_record(&["client", "available", "held", "total", "locked"])?;

    for account in engine.acct_map.iter().map(|(k, v)| (*k, v.available, v.held, v.total, v.locked)) {
        writer.serialize(account)?;
        writer.flush()?;
    }

    Ok(())
}
