use claybrick::writer::Encoder;
use std::{fs::File, io::Write, path::PathBuf};
use structopt::StructOpt;

/// Read a PDF and write it back.
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Input file
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,

    /// Output file
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,
}

pub fn main() {
    env_logger::init();
    let opt = Opt::from_args();

    log::debug!("Read PDF file");
    let pdf = claybrick::read_file(opt.input.as_path());
    let pdf = match pdf {
        Ok(pdf) => pdf,
        Err(e) => {
            log::error!("Error while parsing: {:?}", e);
            return;
        }
    };
    let mut out = Vec::<u8>::new();

    log::debug!("Encode PDF content");
    claybrick::SimpleEncoder::write_to(&pdf, &mut out);

    log::debug!("Write to file");
    let mut buffer = File::create(opt.output).expect("Could not create out file");
    buffer.write_all(&out).expect("Could not write out file");
}
