use std::path::PathBuf;
use structopt::StructOpt;

/// Print the catalog object of the given PDF file.
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
    let pdf = match pdf {
        Ok(pdf) => pdf,
        Err(e) => {
            log::error!("Error while parsing: {:?}", e);
            return;
        }
    };

    println!("Catalog: {:#?}", pdf.catalog());
}
