use nom_tracable::histogram;
use std::path::PathBuf;
use structopt::StructOpt;

/// Read PDF files and print the internal representation.
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Output file
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,
}

pub fn main() {
    let opt = Opt::from_args();

    let pdf = claybrick::read_file(opt.input.as_path()).unwrap();

    histogram();

    println!("{}", pdf);
}
