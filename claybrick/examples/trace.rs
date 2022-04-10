use nom_tracable::histogram;
use std::path::PathBuf;
use structopt::StructOpt;

/// Read PDF files and print the internal representation.
///
/// Trace all steps while parsing the file.
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Output file
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,
}

pub fn main() {
    env_logger::init();
    let opt = Opt::from_args();

    let pdf = claybrick::read_file(opt.input.as_path());
    let _pdf = match pdf {
        Ok(pdf) => pdf,
        Err(e) => {
            log::error!("Error while parsing: {:?}", e);
            return;
        }
    };

    histogram();
}
