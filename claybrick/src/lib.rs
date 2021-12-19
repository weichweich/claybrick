use std::{fs::File, io::Read};

use nom_locate::LocatedSpan;
use nom_tracable::TracableInfo;
use parse::parse;
use pdf::Pdf;

mod parse;
mod pdf;

/// Read a PDF file and return the parsed `Pdf`.
///
/// Panics if the file cannot be read or the PDF cannot get parsed.
/// TODO: don't panic.
pub fn read_file(file_path: &std::path::Path) -> Result<Pdf, ()> {
    let mut input_file = File::open(file_path).unwrap();
    let mut buf = Vec::new();
    input_file.read_to_end(&mut buf).unwrap();

    let info = TracableInfo::new().forward(true).backward(true);
    let span = LocatedSpan::new_extra(&buf[..], info);

    let (_, pdf) = parse(span).unwrap();

    Ok(pdf)
}
