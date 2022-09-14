use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::io::Read;
use csv::{Reader, ReaderBuilder, Trim};

pub fn reader<R>(data: R) -> Reader<R>
    where R: Read
{
    ReaderBuilder::new()
        .trim(Trim::All)
        .flexible(true)
        .from_reader(data)
}

/// Returns the first positional argument sent to this process. If there are no
/// positional arguments, then this returns an error.
///
/// CREDIT: This is taken directly from the `csv` crate tutorial found @ https://docs.rs/csv/latest/csv/tutorial/index.html
pub fn get_first_arg() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from("expected 1 argument, but got none")),
        Some(file_path) => Ok(file_path),
    }
}
