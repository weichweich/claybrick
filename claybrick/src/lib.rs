use std::{fs::File, io::Read};

use error::CbError;
use nom_locate::LocatedSpan;
use nom_tracable::TracableInfo;
use parse::parse_complete;
use pdf::RawPdf;

mod error;
pub mod parse;
mod pdf;
pub mod simple_encode;
pub mod writer;

/// Read a PDF file and return the parsed `Pdf`.
///
/// Panics if the file cannot be read or the PDF cannot get parsed.
/// FIXME: don't panic.
pub fn read_file(file_path: &std::path::Path) -> Result<RawPdf, CbError> {
    let mut input_file = File::open(file_path)?;
    let mut buf = Vec::new();
    input_file.read_to_end(&mut buf)?;

    let info = TracableInfo::new().forward(true).backward(true);
    let span = LocatedSpan::new_extra(&buf[..], info);

    let (_, pdf) = parse_complete(span)?;

    Ok(pdf)
}
