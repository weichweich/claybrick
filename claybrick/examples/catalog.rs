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
    env_logger::init();

    let pdf = claybrick::read_file(opt.input.as_path());
    let pdf = match pdf {
        Ok(pdf) => pdf,
        Err(e) => {
            log::error!("Error while parsing: {:?}", e);
            return;
        }
    };

    println!("Catalog: {:#?}", pdf.catalog());
}
