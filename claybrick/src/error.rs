use std::fmt::Debug;

use crate::parse::error::CbParseError;

#[derive(Debug, Clone)]
pub enum CbError {
    Parse,
    Io,
}

impl<I: Debug> From<nom::Err<CbParseError<I>>> for CbError {
    fn from(err: nom::Err<CbParseError<I>>) -> Self {
        log::error!("Parsing failed: {:?}", err);
        CbError::Parse
    }
}

impl From<std::io::Error> for CbError {
    fn from(_: std::io::Error) -> Self {
        CbError::Io
    }
}
