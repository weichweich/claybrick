use claybrick::parse::{error::CbParseError, Span};
use nom_locate::LocatedSpan;
use nom_tracable::{histogram, TracableInfo};
use std::{fs::File, io::Read, path::PathBuf};
use structopt::StructOpt;

/// Print the Xref section of the PDF file.
#[derive(StructOpt, Debug)]
#[structopt(name = "claybrick-xref")]
struct Opt {
    /// Output file
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,
}

pub fn main() {
    env_logger::init();
    let opt = Opt::from_args();

    let mut input_file = File::open(opt.input).unwrap();
    let mut buf = Vec::new();
    input_file.read_to_end(&mut buf).unwrap();
    let info = TracableInfo::new().forward(true).backward(true);
    let input = LocatedSpan::new_extra(&buf[..], info);

    // find start of the xref section
    let (remainder_xref, _) = claybrick::parse::eof_marker_tail(input).unwrap();
    let (_, startxref) = claybrick::parse::startxref_tail(remainder_xref).unwrap();

    let (remainder_xref, _) = nom::bytes::complete::take::<_, _, CbParseError<Span>>(startxref)(input).unwrap();
    let (_, xref) = claybrick::parse::xref(remainder_xref).unwrap();

    histogram();

    println!("{:#?}", xref);
}
