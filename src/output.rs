use std::io::Write;
use csv::Writer;

pub fn writer<W>(out: W) -> Writer<W>
    where W: Write
{
    Writer::from_writer(out)
}
